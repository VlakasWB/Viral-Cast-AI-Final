/// Services module untuk mengorganisir semua service
/// ID: Module untuk mengatur semua service termasuk rate limiting dan batch processing
/// EN: Module to organize all services including rate limiting and batch processing

pub mod google_ads;
pub mod milvus;
pub mod trend_news;
pub mod xendit;

// ID: Service baru untuk mengatasi rate limiting Groq API
// EN: New services to handle Groq API rate limiting
pub mod rate_limiter;
pub mod batch_processor;
pub mod job_scheduler;
// ID: Nonaktifkan sementara modul yang belum stabil untuk kompilasi
// EN: Temporarily disable unstable modules to restore compilation
// ID: Aktifkan kembali modul layanan yang sebelumnya dinonaktifkan.
// EN: Re-enable previously disabled service modules.
// ID: Nonaktifkan modul layanan yang belum siap untuk menghindari error kompilasi.
// EN: Disable unfinished service modules to avoid compilation errors.
// pub mod ingredient_scheduler;
// pub mod ingredient_prediction_service;

// ID: Re-export untuk kemudahan penggunaan
// EN: Re-exports for ease of use
pub use rate_limiter::GroqRateLimiter;
pub use batch_processor::{BatchProcessor, BatchConfig};
pub use job_scheduler::{JobScheduler, ScheduledJob, JobResult};