use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Rate limiter untuk Groq API
/// ID: Mengontrol jumlah request dan token usage per menit untuk mencegah rate limit
/// EN: Controls request count and token usage per minute to prevent rate limiting
#[derive(Debug, Clone)]
pub struct GroqRateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
}

#[derive(Debug)]
struct RateLimiterInner {
    // ID: Tracking request per menit
    // EN: Request tracking per minute
    requests: Vec<Instant>,

    // ID: Tracking token usage per menit
    // EN: Token usage tracking per minute
    tokens: Vec<(Instant, u32)>,

    // ID: Konfigurasi limit
    // EN: Limit configuration
    max_requests_per_minute: usize,
    max_tokens_per_minute: u32,

    // ID: Cache untuk hasil API
    // EN: Cache for API results
    cache: HashMap<String, CachedResult>,
}

#[derive(Debug, Clone)]
struct CachedResult {
    result: String,
    created_at: Instant,
    ttl: Duration,
}

impl GroqRateLimiter {
    /// ID: Membuat rate limiter baru dengan konfigurasi default
    /// EN: Create new rate limiter with default configuration
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RateLimiterInner {
                requests: Vec::new(),
                tokens: Vec::new(),
                max_requests_per_minute: 30, // Conservative limit
                max_tokens_per_minute: 5000, // Below Groq's 6000 TPM limit
                cache: HashMap::new(),
            })),
        }
    }

    /// ID: Membuat rate limiter dengan konfigurasi kustom
    /// EN: Create rate limiter with custom configuration
    pub fn with_limits(max_requests_per_minute: usize, max_tokens_per_minute: u32) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RateLimiterInner {
                requests: Vec::new(),
                tokens: Vec::new(),
                max_requests_per_minute,
                max_tokens_per_minute,
                cache: HashMap::new(),
            })),
        }
    }

    /// ID: Cek apakah request dapat dilakukan dan estimasi token yang dibutuhkan
    /// EN: Check if request can be made with estimated token usage
    pub async fn can_make_request(&self, estimated_tokens: u32) -> bool {
        let mut inner = self.inner.write().await;

        // ID: Bersihkan data lama (lebih dari 1 menit)
        // EN: Clean old data (older than 1 minute)
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        inner.requests.retain(|&time| time > one_minute_ago);
        inner.tokens.retain(|(time, _)| *time > one_minute_ago);

        // ID: Hitung usage saat ini
        // EN: Calculate current usage
        let current_requests = inner.requests.len();
        let current_tokens: u32 = inner.tokens.iter().map(|(_, tokens)| *tokens).sum();

        // ID: Cek apakah masih dalam limit
        // EN: Check if still within limits
        let can_request = current_requests < inner.max_requests_per_minute;
        let can_tokens = (current_tokens + estimated_tokens) <= inner.max_tokens_per_minute;

        let result = can_request && can_tokens;

        if !result {
            warn!(
                current_requests,
                max_requests = inner.max_requests_per_minute,
                current_tokens,
                estimated_tokens,
                max_tokens = inner.max_tokens_per_minute,
                "Rate limit check failed"
            );
        }

        result
    }

    /// ID: Catat penggunaan request dan token
    /// EN: Record request and token usage
    pub async fn record_usage(&self, tokens_used: u32) {
        let mut inner = self.inner.write().await;
        let now = Instant::now();

        inner.requests.push(now);
        inner.tokens.push((now, tokens_used));

        info!(
            tokens_used,
            total_requests = inner.requests.len(),
            total_tokens = inner.tokens.iter().map(|(_, t)| *t).sum::<u32>(),
            "Recorded Groq API usage"
        );
    }

    /// ID: Dapatkan waktu tunggu yang disarankan jika rate limit tercapai
    /// EN: Get recommended wait time if rate limit is reached
    pub async fn get_wait_time(&self) -> Duration {
        let inner = self.inner.read().await;
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        // ID: Cari request tertua dalam 1 menit terakhir
        // EN: Find oldest request within last minute
        let oldest_request = inner
            .requests
            .iter()
            .filter(|&&time| time > one_minute_ago)
            .min()
            .copied();

        let oldest_token = inner
            .tokens
            .iter()
            .filter(|(time, _)| *time > one_minute_ago)
            .map(|(time, _)| *time)
            .min();

        let oldest = match (oldest_request, oldest_token) {
            (Some(req), Some(tok)) => Some(req.min(tok)),
            (Some(req), None) => Some(req),
            (None, Some(tok)) => Some(tok),
            (None, None) => None,
        };

        match oldest {
            Some(oldest_time) => {
                let elapsed = now.duration_since(oldest_time);
                if elapsed < Duration::from_secs(60) {
                    Duration::from_secs(60) - elapsed + Duration::from_secs(5) // Extra 5s buffer
                } else {
                    Duration::from_secs(5) // Minimum wait
                }
            }
            None => Duration::from_secs(5),
        }
    }

    /// ID: Simpan hasil ke cache
    /// EN: Store result in cache
    pub async fn cache_result(&self, key: String, result: String, ttl: Duration) {
        let mut inner = self.inner.write().await;
        inner.cache.insert(
            key,
            CachedResult {
                result,
                created_at: Instant::now(),
                ttl,
            },
        );
    }

    /// ID: Ambil hasil dari cache jika masih valid
    /// EN: Get result from cache if still valid
    pub async fn get_cached_result(&self, key: &str) -> Option<String> {
        let mut inner = self.inner.write().await;

        // ID: Bersihkan cache yang expired
        // EN: Clean expired cache entries
        let now = Instant::now();
        inner
            .cache
            .retain(|_, cached| now.duration_since(cached.created_at) < cached.ttl);

        inner.cache.get(key).map(|cached| cached.result.clone())
    }

    /// ID: Dapatkan statistik penggunaan saat ini
    /// EN: Get current usage statistics
    pub async fn get_usage_stats(&self) -> UsageStats {
        let inner = self.inner.read().await;
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        let current_requests = inner
            .requests
            .iter()
            .filter(|&&time| time > one_minute_ago)
            .count();

        let current_tokens: u32 = inner
            .tokens
            .iter()
            .filter(|(time, _)| *time > one_minute_ago)
            .map(|(_, tokens)| *tokens)
            .sum();

        UsageStats {
            current_requests,
            max_requests: inner.max_requests_per_minute,
            current_tokens,
            max_tokens: inner.max_tokens_per_minute,
            cache_size: inner.cache.len(),
        }
    }
}

#[derive(Debug)]
pub struct UsageStats {
    pub current_requests: usize,
    pub max_requests: usize,
    pub current_tokens: u32,
    pub max_tokens: u32,
    pub cache_size: usize,
}

impl Default for GroqRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = GroqRateLimiter::with_limits(2, 1000);

        // ID: Request pertama harus berhasil
        // EN: First request should succeed
        assert!(limiter.can_make_request(100).await);
        limiter.record_usage(100).await;

        // ID: Request kedua harus berhasil
        // EN: Second request should succeed
        assert!(limiter.can_make_request(100).await);
        limiter.record_usage(100).await;

        // ID: Request ketiga harus gagal (melebihi limit request)
        // EN: Third request should fail (exceeds request limit)
        assert!(!limiter.can_make_request(100).await);
    }

    #[tokio::test]
    async fn test_token_limit() {
        let limiter = GroqRateLimiter::with_limits(10, 500);

        // ID: Request dengan token tinggi harus gagal
        // EN: Request with high tokens should fail
        assert!(!limiter.can_make_request(600).await);

        // ID: Request dengan token normal harus berhasil
        // EN: Request with normal tokens should succeed
        assert!(limiter.can_make_request(400).await);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let limiter = GroqRateLimiter::new();

        // ID: Cache kosong pada awalnya
        // EN: Cache should be empty initially
        assert!(limiter.get_cached_result("test").await.is_none());

        // ID: Simpan ke cache
        // EN: Store in cache
        limiter
            .cache_result(
                "test".to_string(),
                "result".to_string(),
                Duration::from_secs(60),
            )
            .await;

        // ID: Harus bisa mengambil dari cache
        // EN: Should be able to retrieve from cache
        assert_eq!(
            limiter.get_cached_result("test").await,
            Some("result".to_string())
        );
    }
}
