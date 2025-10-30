use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::services::rate_limiter::GroqRateLimiter;

/// Batch processor untuk memproses item dalam batch kecil dengan rate limiting
/// ID: Memproses item dalam batch kecil untuk menghindari rate limit dan overload
/// EN: Processes items in small batches to avoid rate limits and overload
#[derive(Debug)]
pub struct BatchProcessor<T> {
    inner: Arc<RwLock<BatchProcessorInner<T>>>,
    rate_limiter: GroqRateLimiter,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug)]
struct BatchProcessorInner<T> {
    queue: VecDeque<BatchItem<T>>,
    config: BatchConfig,
    stats: ProcessingStats,
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// ID: Jumlah maksimum item per batch
    /// EN: Maximum number of items per batch
    pub max_items_per_batch: usize,

    /// ID: Interval waktu antar batch (dalam detik)
    /// EN: Interval between batches (in seconds)
    pub batch_interval_secs: u64,

    /// ID: Jumlah maksimum batch yang berjalan bersamaan
    /// EN: Maximum number of concurrent batches
    pub max_concurrent_batches: usize,

    /// ID: Timeout untuk setiap batch (dalam detik)
    /// EN: Timeout for each batch (in seconds)
    pub batch_timeout_secs: u64,

    /// ID: Estimasi token per item untuk rate limiting
    /// EN: Estimated tokens per item for rate limiting
    pub estimated_tokens_per_item: u32,
}

#[derive(Debug, Clone)]
struct BatchItem<T> {
    id: Uuid,
    data: T,
    created_at: Instant,
    attempts: u32,
    max_attempts: u32,
}

// ID: Tambahkan Clone agar dapat dikembalikan melalui get_stats tanpa meminjam hidup terlalu lama.
// EN: Add Clone so get_stats can return a copy without holding the borrow too long.
#[derive(Debug, Default, Clone)]
struct ProcessingStats {
    total_items_queued: u64,
    total_items_processed: u64,
    total_items_failed: u64,
    total_batches_processed: u64,
    current_queue_size: usize,
    last_batch_time: Option<Instant>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_items_per_batch: 3,          // Conservative untuk Groq API
            batch_interval_secs: 3600,       // 1 jam seperti yang diminta user
            max_concurrent_batches: 1,       // Satu batch pada satu waktu
            batch_timeout_secs: 300,         // 5 menit timeout
            estimated_tokens_per_item: 2000, // Estimasi konservatif
        }
    }
}

impl<T> BatchProcessor<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// ID: Membuat batch processor baru
    /// EN: Create new batch processor
    pub fn new(config: BatchConfig, rate_limiter: GroqRateLimiter) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_batches));

        Self {
            inner: Arc::new(RwLock::new(BatchProcessorInner {
                queue: VecDeque::new(),
                config,
                stats: ProcessingStats::default(),
            })),
            rate_limiter,
            semaphore,
        }
    }

    /// ID: Tambahkan item ke queue untuk diproses
    /// EN: Add item to queue for processing
    pub async fn enqueue_item(&self, data: T) -> Uuid {
        let mut inner = self.inner.write().await;
        let item_id = Uuid::new_v4();

        let item = BatchItem {
            id: item_id,
            data,
            created_at: Instant::now(),
            attempts: 0,
            max_attempts: 3,
        };

        inner.queue.push_back(item);
        inner.stats.total_items_queued += 1;
        inner.stats.current_queue_size = inner.queue.len();

        info!(
            item_id = %item_id,
            queue_size = inner.queue.len(),
            "Item added to batch queue"
        );

        item_id
    }

    /// ID: Tambahkan beberapa item sekaligus ke queue
    /// EN: Add multiple items to queue at once
    pub async fn enqueue_items(&self, items: Vec<T>) -> Vec<Uuid> {
        let mut inner = self.inner.write().await;
        let mut item_ids = Vec::new();

        for data in items {
            let item_id = Uuid::new_v4();
            let item = BatchItem {
                id: item_id,
                data,
                created_at: Instant::now(),
                attempts: 0,
                max_attempts: 3,
            };

            inner.queue.push_back(item);
            inner.stats.total_items_queued += 1;
            item_ids.push(item_id);
        }

        inner.stats.current_queue_size = inner.queue.len();

        info!(
            items_count = item_ids.len(),
            queue_size = inner.queue.len(),
            "Multiple items added to batch queue"
        );

        item_ids
    }

    /// ID: Proses batch berikutnya jika memungkinkan
    /// EN: Process next batch if possible
    pub async fn process_next_batch<F, Fut, R>(&self, processor_fn: F) -> Result<Vec<R>, String>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<Vec<R>, String>> + Send,
        R: Send + Sync,
    {
        // ID: Dapatkan permit untuk concurrent processing
        // EN: Acquire permit for concurrent processing
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire semaphore: {}", e))?;

        let batch = {
            let mut inner = self.inner.write().await;

            // ID: Cek apakah sudah waktunya untuk batch berikutnya
            // EN: Check if it's time for next batch
            if let Some(last_batch_time) = inner.stats.last_batch_time {
                let elapsed = Instant::now().duration_since(last_batch_time);
                let required_interval = Duration::from_secs(inner.config.batch_interval_secs);

                if elapsed < required_interval {
                    let wait_time = required_interval - elapsed;
                    info!(
                        wait_seconds = wait_time.as_secs(),
                        "Waiting for batch interval"
                    );
                    drop(inner); // Release lock before sleeping
                    sleep(wait_time).await;
                    inner = self.inner.write().await;
                }
            }

            // ID: Ambil item untuk batch
            // EN: Take items for batch
            let batch_size = inner.config.max_items_per_batch.min(inner.queue.len());
            if batch_size == 0 {
                return Ok(Vec::new());
            }

            let mut batch_items = Vec::new();
            for _ in 0..batch_size {
                if let Some(item) = inner.queue.pop_front() {
                    batch_items.push(item);
                }
            }

            inner.stats.current_queue_size = inner.queue.len();
            batch_items
        };

        if batch.is_empty() {
            return Ok(Vec::new());
        }

        // ID: Cek rate limit sebelum memproses
        // EN: Check rate limit before processing
        let estimated_tokens = batch.len() as u32 * {
            let inner = self.inner.read().await;
            inner.config.estimated_tokens_per_item
        };

        if !self.rate_limiter.can_make_request(estimated_tokens).await {
            // ID: Rate limit tercapai, kembalikan item ke queue
            // EN: Rate limit reached, return items to queue
            let mut inner = self.inner.write().await;
            for item in batch.into_iter().rev() {
                inner.queue.push_front(item);
            }
            inner.stats.current_queue_size = inner.queue.len();

            let wait_time = self.rate_limiter.get_wait_time().await;
            warn!(
                wait_seconds = wait_time.as_secs(),
                "Rate limit reached, waiting before retry"
            );

            return Err(format!(
                "Rate limit reached, please wait {} seconds",
                wait_time.as_secs()
            ));
        }

        // ID: Proses batch
        // EN: Process batch
        let batch_data: Vec<T> = batch.iter().map(|item| item.data.clone()).collect();
        let batch_ids: Vec<Uuid> = batch.iter().map(|item| item.id).collect();

        info!(
            batch_size = batch.len(),
            batch_ids = ?batch_ids,
            "Processing batch"
        );

        let start_time = Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs({
                let inner = self.inner.read().await;
                inner.config.batch_timeout_secs
            }),
            processor_fn(batch_data),
        )
        .await;

        let processing_time = start_time.elapsed();

        match result {
            Ok(Ok(results)) => {
                // ID: Batch berhasil diproses
                // EN: Batch processed successfully
                self.rate_limiter.record_usage(estimated_tokens).await;

                let mut inner = self.inner.write().await;
                inner.stats.total_items_processed += batch.len() as u64;
                inner.stats.total_batches_processed += 1;
                inner.stats.last_batch_time = Some(Instant::now());

                info!(
                    batch_size = batch.len(),
                    processing_time_ms = processing_time.as_millis(),
                    "Batch processed successfully"
                );

                Ok(results)
            }
            Ok(Err(e)) => {
                // ID: Error dalam pemrosesan, coba lagi jika masih ada attempts
                // EN: Processing error, retry if attempts remaining
                error!(
                    error = %e,
                    batch_size = batch.len(),
                    "Batch processing failed"
                );

                self.handle_batch_failure(batch, e).await
            }
            Err(_) => {
                // ID: Timeout
                // EN: Timeout occurred
                let error_msg = format!(
                    "Batch processing timeout after {} seconds",
                    processing_time.as_secs()
                );
                error!(
                    timeout_secs = processing_time.as_secs(),
                    batch_size = batch.len(),
                    "Batch processing timeout"
                );

                self.handle_batch_failure(batch, error_msg).await
            }
        }
    }

    /// ID: Tangani kegagalan batch dengan retry logic
    /// EN: Handle batch failure with retry logic
    async fn handle_batch_failure<R>(
        &self,
        mut batch: Vec<BatchItem<T>>,
        error: String,
    ) -> Result<Vec<R>, String> {
        let mut inner = self.inner.write().await;

        // ID: Increment attempts dan re-queue item yang masih bisa di-retry
        // EN: Increment attempts and re-queue items that can still be retried
        let mut failed_items = 0;

        for item in &mut batch {
            item.attempts += 1;

            if item.attempts < item.max_attempts {
                // ID: Masih bisa di-retry, kembalikan ke queue
                // EN: Can still retry, return to queue
                inner.queue.push_back(item.clone());
            } else {
                // ID: Sudah mencapai max attempts, anggap gagal
                // EN: Reached max attempts, consider failed
                failed_items += 1;
                inner.stats.total_items_failed += 1;
            }
        }

        inner.stats.current_queue_size = inner.queue.len();

        warn!(
            failed_items,
            requeued_items = batch.len() - failed_items,
            error = %error,
            "Batch failure handled"
        );

        Err(error)
    }

    /// ID: Dapatkan statistik pemrosesan
    /// EN: Get processing statistics
    pub async fn get_stats(&self) -> ProcessingStats {
        let inner = self.inner.read().await;
        inner.stats.clone()
    }

    /// ID: Dapatkan ukuran queue saat ini
    /// EN: Get current queue size
    pub async fn get_queue_size(&self) -> usize {
        let inner = self.inner.read().await;
        inner.queue.len()
    }

    /// ID: Bersihkan queue (hapus semua item)
    /// EN: Clear queue (remove all items)
    pub async fn clear_queue(&self) -> usize {
        let mut inner = self.inner.write().await;
        let cleared_count = inner.queue.len();
        inner.queue.clear();
        inner.stats.current_queue_size = 0;

        info!(cleared_items = cleared_count, "Queue cleared");

        cleared_count
    }

    /// ID: Update konfigurasi batch
    /// EN: Update batch configuration
    pub async fn update_config(&self, new_config: BatchConfig) {
        let mut inner = self.inner.write().await;
        inner.config = new_config;

        info!("Batch configuration updated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rate_limiter::GroqRateLimiter;

    #[tokio::test]
    async fn test_batch_processor_basic() {
        let config = BatchConfig {
            max_items_per_batch: 2,
            batch_interval_secs: 1,
            max_concurrent_batches: 1,
            batch_timeout_secs: 10,
            estimated_tokens_per_item: 100,
        };

        let rate_limiter = GroqRateLimiter::with_limits(10, 1000);
        let processor = BatchProcessor::new(config, rate_limiter);

        // ID: Tambahkan item ke queue
        // EN: Add items to queue
        let id1 = processor.enqueue_item("item1".to_string()).await;
        let id2 = processor.enqueue_item("item2".to_string()).await;

        assert_eq!(processor.get_queue_size().await, 2);

        // ID: Proses batch
        // EN: Process batch
        let result = processor
            .process_next_batch(
                |items| async move { Ok(items.into_iter().map(|s| s.len()).collect()) },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(processor.get_queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_batch_processor_multiple_items() {
        let config = BatchConfig::default();
        let rate_limiter = GroqRateLimiter::new();
        let processor = BatchProcessor::new(config, rate_limiter);

        // ID: Tambahkan beberapa item sekaligus
        // EN: Add multiple items at once
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let ids = processor.enqueue_items(items).await;

        assert_eq!(ids.len(), 3);
        assert_eq!(processor.get_queue_size().await, 3);
    }
}
