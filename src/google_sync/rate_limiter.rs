//! Token bucket rate limiter for Google API quotas
//!
//! Prevents hitting Google's rate limits:
//! - Gmail: ~250 quota units/sec/user
//! - Drive: ~1000 queries/100 sec/user
//!
//! Uses a simple token bucket algorithm with async waiting.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

/// Token bucket rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<RateLimiterInner>>,
}

struct RateLimiterInner {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a rate limiter for Gmail API (conservative: 50 req/sec)
    pub fn gmail() -> Self {
        Self::new(50.0, 50.0)
    }

    /// Create a rate limiter for Drive API (conservative: 10 req/sec)
    pub fn drive() -> Self {
        Self::new(10.0, 10.0)
    }

    /// Create with custom capacity and refill rate
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiterInner {
                tokens: max_tokens,
                max_tokens,
                refill_rate,
                last_refill: Instant::now(),
            })),
        }
    }

    /// Wait until a token is available, then consume one
    pub async fn acquire(&self) {
        loop {
            {
                let mut inner = self.inner.lock().await;
                inner.refill();
                if inner.tokens >= 1.0 {
                    inner.tokens -= 1.0;
                    return;
                }
            }
            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Try to acquire without waiting. Returns true if token was available.
    pub async fn try_acquire(&self) -> bool {
        let mut inner = self.inner.lock().await;
        inner.refill();
        if inner.tokens >= 1.0 {
            inner.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get current token count (for diagnostics)
    pub async fn available_tokens(&self) -> f64 {
        let mut inner = self.inner.lock().await;
        inner.refill();
        inner.tokens
    }
}

impl RateLimiterInner {
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(5.0, 5.0);

        // Should be able to acquire 5 tokens immediately
        for _ in 0..5 {
            assert!(limiter.try_acquire().await);
        }

        // 6th should fail (bucket empty)
        assert!(!limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(2.0, 100.0); // Fast refill for testing

        // Drain tokens
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await);

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should have tokens again
        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_gmail_limiter() {
        let limiter = RateLimiter::gmail();
        let tokens = limiter.available_tokens().await;
        assert!((tokens - 50.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_drive_limiter() {
        let limiter = RateLimiter::drive();
        let tokens = limiter.available_tokens().await;
        assert!((tokens - 10.0).abs() < 1.0);
    }
}
