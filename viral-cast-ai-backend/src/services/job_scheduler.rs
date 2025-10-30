use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Job scheduler untuk menjalankan tugas secara berkala
/// ID: Menjalankan tugas batch processing secara otomatis dengan interval tertentu
/// EN: Runs batch processing tasks automatically at specified intervals
#[derive(Debug)]
pub struct JobScheduler {
    inner: Arc<RwLock<JobSchedulerInner>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

#[derive(Debug)]
struct JobSchedulerInner {
    jobs: HashMap<Uuid, ScheduledJob>,
    is_running: bool,
    stats: SchedulerStats,
}

#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub name: String,
    pub interval: Duration,
    pub last_run: Option<Instant>,
    pub next_run: Instant,
    pub enabled: bool,
    pub run_count: u64,
    pub last_duration: Option<Duration>,
    pub last_result: Option<JobResult>,
}

#[derive(Debug, Clone)]
pub enum JobResult {
    Success(String),
    Error(String),
}

#[derive(Debug, Default, Clone)]
pub struct SchedulerStats {
    pub total_jobs: usize,
    pub active_jobs: usize,
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub last_run_time: Option<Instant>,
}

pub type JobFunction = Box<
    dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
        + Send
        + Sync,
>;

impl JobScheduler {
    /// ID: Membuat job scheduler baru
    /// EN: Create new job scheduler
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(JobSchedulerInner {
                jobs: HashMap::new(),
                is_running: false,
                stats: SchedulerStats::default(),
            })),
            shutdown_tx: None,
        }
    }

    /// ID: Tambahkan job baru ke scheduler
    /// EN: Add new job to scheduler
    pub async fn add_job<F, Fut>(&mut self, name: String, interval: Duration, job_fn: F) -> Uuid
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String, String>> + Send + 'static,
    {
        let job_id = Uuid::new_v4();
        let now = Instant::now();

        let job = ScheduledJob {
            id: job_id,
            name: name.clone(),
            interval,
            last_run: None,
            next_run: now + interval,
            enabled: true,
            run_count: 0,
            last_duration: None,
            last_result: None,
        };

        let mut inner = self.inner.write().await;
        inner.jobs.insert(job_id, job);
        inner.stats.total_jobs = inner.jobs.len();
        inner.stats.active_jobs = inner.jobs.values().filter(|j| j.enabled).count();

        info!(
            job_id = %job_id,
            job_name = %name,
            interval_secs = interval.as_secs(),
            "Job added to scheduler"
        );

        job_id
    }

    /// ID: Mulai menjalankan scheduler
    /// EN: Start running the scheduler
    pub async fn start(&mut self) -> Result<(), String> {
        let mut inner = self.inner.write().await;

        if inner.is_running {
            return Err("Scheduler is already running".to_string());
        }

        inner.is_running = true;
        drop(inner);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let scheduler_inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(10)); // Check every 10 seconds

            info!("Job scheduler started");

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if let Err(e) = Self::run_pending_jobs(&scheduler_inner).await {
                            error!(error = %e, "Error running pending jobs");
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Job scheduler shutdown signal received");
                        break;
                    }
                }
            }

            let mut inner = scheduler_inner.write().await;
            inner.is_running = false;
            info!("Job scheduler stopped");
        });

        Ok(())
    }

    /// ID: Hentikan scheduler
    /// EN: Stop the scheduler
    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx
                .send(())
                .await
                .map_err(|e| format!("Failed to send shutdown signal: {}", e))?;
        }

        // ID: Tunggu sampai scheduler benar-benar berhenti
        // EN: Wait until scheduler actually stops
        let mut attempts = 0;
        while attempts < 50 {
            // Max 5 seconds wait
            {
                let inner = self.inner.read().await;
                if !inner.is_running {
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }

        Ok(())
    }

    /// ID: Jalankan job yang sudah waktunya
    /// EN: Run jobs that are due
    async fn run_pending_jobs(
        scheduler_inner: &Arc<RwLock<JobSchedulerInner>>,
    ) -> Result<(), String> {
        let jobs_to_run = {
            let inner = scheduler_inner.read().await;
            let now = Instant::now();

            inner
                .jobs
                .values()
                .filter(|job| job.enabled && job.next_run <= now)
                .map(|job| job.clone())
                .collect::<Vec<_>>()
        };

        for job in jobs_to_run {
            if let Err(e) = Self::execute_job(&job, scheduler_inner).await {
                error!(
                    job_id = %job.id,
                    job_name = %job.name,
                    error = %e,
                    "Failed to execute job"
                );
            }
        }

        Ok(())
    }

    /// ID: Eksekusi job individual
    /// EN: Execute individual job
    async fn execute_job(
        job: &ScheduledJob,
        scheduler_inner: &Arc<RwLock<JobSchedulerInner>>,
    ) -> Result<(), String> {
        let start_time = Instant::now();

        info!(
            job_id = %job.id,
            job_name = %job.name,
            "Executing job"
        );

        // ID: Untuk demo, kita akan membuat job sederhana
        // Dalam implementasi nyata, ini akan memanggil batch processor
        // EN: For demo, we'll create a simple job
        // In real implementation, this would call the batch processor
        let result = Self::dummy_job_execution(&job.name).await;

        let duration = start_time.elapsed();
        let now = Instant::now();

        // ID: Update job statistics
        // EN: Update job statistics
        {
            let mut inner = scheduler_inner.write().await;
            if let Some(job_mut) = inner.jobs.get_mut(&job.id) {
                job_mut.last_run = Some(now);
                job_mut.next_run = now + job_mut.interval;
                job_mut.run_count += 1;
                job_mut.last_duration = Some(duration);
                job_mut.last_result = Some(result.clone());
            }

            inner.stats.total_runs += 1;
            inner.stats.last_run_time = Some(now);

            match &result {
                JobResult::Success(_) => {
                    inner.stats.successful_runs += 1;
                }
                JobResult::Error(_) => {
                    inner.stats.failed_runs += 1;
                }
            }
        }

        match result {
            JobResult::Success(msg) => {
                info!(
                    job_id = %job.id,
                    job_name = %job.name,
                    duration_ms = duration.as_millis(),
                    result = %msg,
                    "Job executed successfully"
                );
                Ok(())
            }
            JobResult::Error(err) => {
                warn!(
                    job_id = %job.id,
                    job_name = %job.name,
                    duration_ms = duration.as_millis(),
                    error = %err,
                    "Job execution failed"
                );
                Err(err)
            }
        }
    }

    /// ID: Dummy job execution untuk demo
    /// EN: Dummy job execution for demo
    async fn dummy_job_execution(job_name: &str) -> JobResult {
        // ID: Simulasi pemrosesan
        // EN: Simulate processing
        sleep(Duration::from_millis(100)).await;

        if job_name.contains("ingredient") {
            JobResult::Success(format!("Processed ingredient predictions for {}", job_name))
        } else {
            JobResult::Success(format!("Executed job: {}", job_name))
        }
    }

    /// ID: Dapatkan status job tertentu
    /// EN: Get status of specific job
    pub async fn get_job_status(&self, job_id: Uuid) -> Option<ScheduledJob> {
        let inner = self.inner.read().await;
        inner.jobs.get(&job_id).cloned()
    }

    /// ID: Dapatkan semua job
    /// EN: Get all jobs
    pub async fn get_all_jobs(&self) -> Vec<ScheduledJob> {
        let inner = self.inner.read().await;
        inner.jobs.values().cloned().collect()
    }

    /// ID: Enable/disable job
    /// EN: Enable/disable job
    pub async fn set_job_enabled(&self, job_id: Uuid, enabled: bool) -> Result<(), String> {
        let mut inner = self.inner.write().await;

        // ID: Ubah status job terlebih dahulu lalu akhiri borrow mutable sebelum menghitung aktif
        // EN: Toggle job status first, then end mutable borrow before counting active jobs
        if let Some(job) = inner.jobs.get_mut(&job_id) {
            job.enabled = enabled;
        } else {
            return Err(format!("Job with ID {} not found", job_id));
        }

        // ID: Setelah borrow mutable selesai, aman menghitung jumlah job aktif
        // EN: After mutable borrow ends, safely compute active job count
        inner.stats.active_jobs = inner.jobs.values().filter(|j| j.enabled).count();

        // ID: Logging perubahan status
        // EN: Log status change
        if let Some(job) = inner.jobs.get(&job_id) {
            info!(
                job_id = %job_id,
                job_name = %job.name,
                enabled = enabled,
                "Job status changed"
            );
        }

        Ok(())
    }

    /// ID: Hapus job dari scheduler
    /// EN: Remove job from scheduler
    pub async fn remove_job(&self, job_id: Uuid) -> Result<(), String> {
        let mut inner = self.inner.write().await;

        if inner.jobs.remove(&job_id).is_some() {
            inner.stats.total_jobs = inner.jobs.len();
            inner.stats.active_jobs = inner.jobs.values().filter(|j| j.enabled).count();

            info!(
                job_id = %job_id,
                "Job removed from scheduler"
            );

            Ok(())
        } else {
            Err(format!("Job with ID {} not found", job_id))
        }
    }

    /// ID: Jalankan job secara manual (tidak menunggu jadwal)
    /// EN: Run job manually (don't wait for schedule)
    pub async fn run_job_now(&self, job_id: Uuid) -> Result<JobResult, String> {
        let job = {
            let inner = self.inner.read().await;
            inner
                .jobs
                .get(&job_id)
                .cloned()
                .ok_or_else(|| format!("Job with ID {} not found", job_id))?
        };

        let start_time = Instant::now();
        let result = Self::dummy_job_execution(&job.name).await;
        let duration = start_time.elapsed();

        // ID: Update statistik tanpa mengubah jadwal berikutnya
        // EN: Update statistics without changing next schedule
        {
            let mut inner = self.inner.write().await;
            if let Some(job_mut) = inner.jobs.get_mut(&job_id) {
                job_mut.last_run = Some(Instant::now());
                job_mut.run_count += 1;
                job_mut.last_duration = Some(duration);
                job_mut.last_result = Some(result.clone());
            }

            inner.stats.total_runs += 1;
            match &result {
                JobResult::Success(_) => inner.stats.successful_runs += 1,
                JobResult::Error(_) => inner.stats.failed_runs += 1,
            }
        }

        info!(
            job_id = %job_id,
            job_name = %job.name,
            duration_ms = duration.as_millis(),
            "Job executed manually"
        );

        Ok(result)
    }

    /// ID: Dapatkan statistik scheduler
    /// EN: Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let inner = self.inner.read().await;
        inner.stats.clone()
    }

    /// ID: Cek apakah scheduler sedang berjalan
    /// EN: Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        let inner = self.inner.read().await;
        inner.is_running
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_job_scheduler_basic() {
        let mut scheduler = JobScheduler::new();

        // ID: Tambahkan job dengan interval pendek untuk testing
        // EN: Add job with short interval for testing
        let job_id = scheduler
            .add_job(
                "test_job".to_string(),
                Duration::from_millis(100),
                || async { Ok("Test completed".to_string()) },
            )
            .await;

        // ID: Cek job ditambahkan
        // EN: Check job was added
        let job = scheduler.get_job_status(job_id).await;
        assert!(job.is_some());
        assert_eq!(job.unwrap().name, "test_job");
    }

    #[tokio::test]
    async fn test_job_scheduler_manual_run() {
        let mut scheduler = JobScheduler::new();

        let job_id = scheduler
            .add_job(
                "manual_test".to_string(),
                Duration::from_secs(3600), // 1 hour
                || async { Ok("Manual test completed".to_string()) },
            )
            .await;

        // ID: Jalankan job secara manual
        // EN: Run job manually
        let result = scheduler.run_job_now(job_id).await;
        assert!(result.is_ok());

        match result.unwrap() {
            JobResult::Success(msg) => assert!(msg.contains("manual_test")),
            JobResult::Error(_) => panic!("Expected success"),
        }
    }

    #[tokio::test]
    async fn test_job_enable_disable() {
        let mut scheduler = JobScheduler::new();

        let job_id = scheduler
            .add_job(
                "toggle_test".to_string(),
                Duration::from_secs(60),
                || async { Ok("Toggle test".to_string()) },
            )
            .await;

        // ID: Disable job
        // EN: Disable job
        scheduler.set_job_enabled(job_id, false).await.unwrap();

        let job = scheduler.get_job_status(job_id).await.unwrap();
        assert!(!job.enabled);

        // ID: Enable job kembali
        // EN: Enable job again
        scheduler.set_job_enabled(job_id, true).await.unwrap();

        let job = scheduler.get_job_status(job_id).await.unwrap();
        assert!(job.enabled);
    }
}
