// ID: Service untuk menjadwalkan prediksi bahan baku secara otomatis
// EN: Service for scheduling ingredient predictions automatically

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::repository::stores as stores_repository;
use crate::repository::store_ingredient_predictions as ingredient_predictions_repository;
use crate::services::rate_limiter::GroqRateLimiter;
use crate::AppState;

pub struct IngredientScheduler {
    state: Arc<AppState>,
    rate_limiter: GroqRateLimiter,
    is_running: bool,
    interval_hours: u64,
}

impl IngredientScheduler {
    pub fn new(state: Arc<AppState>) -> Self {
        let interval_hours = std::env::var("BATCH_PROCESSING_INTERVAL_HOURS")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u64>()
            .unwrap_or(1);

        Self {
            state,
            rate_limiter: GroqRateLimiter::new(),
            is_running: false,
            interval_hours,
        }
    }

    pub async fn start(&mut self) {
        if self.is_running {
            warn!("Ingredient scheduler is already running");
            return;
        }

        self.is_running = true;
        info!(
            interval_hours = self.interval_hours,
            "Starting ingredient prediction scheduler"
        );

        let mut interval_timer = interval(Duration::from_secs(self.interval_hours * 3600));

        loop {
            if !self.is_running {
                break;
            }

            interval_timer.tick().await;

            if let Err(e) = self.process_scheduled_predictions().await {
                error!(error = %e, "Failed to process scheduled ingredient predictions");
            }
        }

        info!("Ingredient scheduler stopped");
    }

    pub fn stop(&mut self) {
        info!("Stopping ingredient scheduler");
        self.is_running = false;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    async fn process_scheduled_predictions(&self) -> Result<(), String> {
        info!("Processing scheduled ingredient predictions");

        // ID: Ambil semua store yang aktif
        // EN: Get all active stores
        let stores = stores_repository::get_all_active_stores(&self.state.db)
            .await
            .map_err(|e| format!("Failed to get active stores: {}", e))?;

        if stores.is_empty() {
            info!("No active stores found for scheduled predictions");
            return Ok(());
        }

        info!(store_count = stores.len(), "Processing predictions for stores");

        // ID: Proses setiap store dengan delay untuk menghindari rate limit
        // EN: Process each store with delay to avoid rate limit
        for (index, store) in stores.iter().enumerate() {
            if !self.is_running {
                break;
            }

            // ID: Tambahkan delay antar store (kecuali store pertama)
            // EN: Add delay between stores (except first store)
            if index > 0 {
                let store_delay_secs = std::env::var("GROQ_STORE_DELAY_SECS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse::<u64>()
                    .unwrap_or(30);

                info!(
                    store_uuid = %store.uuid,
                    delay_secs = store_delay_secs,
                    "Waiting between stores to respect rate limits"
                );

                sleep(Duration::from_secs(store_delay_secs)).await;
            }

            if let Err(e) = self.process_store_predictions(store.uuid).await {
                error!(
                    store_uuid = %store.uuid,
                    error = %e,
                    "Failed to process predictions for store"
                );
                // ID: Lanjutkan ke store berikutnya meskipun ada error
                // EN: Continue to next store even if there's an error
                continue;
            }

            info!(
                store_uuid = %store.uuid,
                "Successfully processed predictions for store"
            );
        }

        info!("Completed scheduled ingredient predictions processing");
        Ok(())
    }

    async fn process_store_predictions(&self, store_uuid: Uuid) -> Result<(), String> {
        // ID: Cek apakah rate limiter mengizinkan request
        // EN: Check if rate limiter allows request
        let estimated_tokens = 2000; // Rough estimate for ingredient prediction
        
        if !self.rate_limiter.can_make_request(estimated_tokens).await {
            let wait_time = self.rate_limiter.get_suggested_wait_time().await;
            
            if wait_time > 300 { // ID: Jika harus menunggu lebih dari 5 menit, skip
                warn!(
                    store_uuid = %store_uuid,
                    wait_time_secs = wait_time,
                    "Skipping store due to long rate limit wait time"
                );
                return Ok(());
            }

            info!(
                store_uuid = %store_uuid,
                wait_time_secs = wait_time,
                "Waiting for rate limit before processing store"
            );
            
            sleep(Duration::from_secs(wait_time)).await;
        }

        // ID: Proses prediksi untuk store ini
        // EN: Process predictions for this store
        // Note: Ini akan memanggil logic yang sama dengan handler API
        // This would call the same logic as the API handler
        
        // ID: Untuk implementasi lengkap, kita perlu extract logic dari handler
        // EN: For complete implementation, we need to extract logic from handler
        // ke dalam service terpisah yang bisa dipanggil dari scheduler dan handler
        // into separate service that can be called from both scheduler and handler
        
        info!(
            store_uuid = %store_uuid,
            "Processing ingredient predictions for store (placeholder)"
        );

        // ID: Catat penggunaan token
        // EN: Record token usage
        self.rate_limiter.record_request(estimated_tokens).await;

        Ok(())
    }

    pub async fn get_stats(&self) -> SchedulerStats {
        SchedulerStats {
            is_running: self.is_running,
            interval_hours: self.interval_hours,
            rate_limiter_stats: self.rate_limiter.get_stats().await,
        }
    }
}

#[derive(Debug)]
pub struct SchedulerStats {
    pub is_running: bool,
    pub interval_hours: u64,
    pub rate_limiter_stats: crate::services::rate_limiter::RateLimiterStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        // ID: Mock AppState untuk testing
        // EN: Mock AppState for testing
        // Note: Ini memerlukan setup database mock untuk test lengkap
        // This requires mock database setup for complete testing
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        // ID: Test start dan stop scheduler
        // EN: Test scheduler start and stop
    }
}