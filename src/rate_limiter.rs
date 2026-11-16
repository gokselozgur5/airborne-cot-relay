use std::time::Instant;

/// Configuration for a token bucket rate limiter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateLimiterConfig {
    pub capacity: f64,
    pub refill_rate: f64,
}

/// Statistics collected by the rate limiter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RateLimiterStats {
    pub allowed: u64,
    pub dropped: u64,
}

impl RateLimiterConfig {
    pub const fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
        }
    }
}

/// Token bucket filter for smoothing bursty network traffic.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    config: RateLimiterConfig,
    tokens: f64,
    last_refill: Instant,
    stats: RateLimiterStats,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self::with_config(RateLimiterConfig::new(capacity, refill_rate))
    }

    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self {
            tokens: config.capacity,
            last_refill: Instant::now(),
            stats: RateLimiterStats::default(),
            config,
        }
    }
}

impl TokenBucket {
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.config.refill_rate).min(self.config.capacity);
        self.last_refill = now;
    }
}

impl TokenBucket {
    pub fn try_acquire(&mut self, amount: f64) -> bool {
        self.refill();
        if self.tokens >= amount {
            self.tokens -= amount;
            self.stats.allowed += 1;
            true
        } else {
            self.stats.dropped += 1;
            false
        }
    }
}

impl TokenBucket {
    pub fn config(&self) -> RateLimiterConfig {
        self.config
    }

    pub fn stats(&self) -> RateLimiterStats {
        self.stats
    }

    pub fn tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

impl TokenBucket {
    pub fn reset(&mut self) {
        self.tokens = self.config.capacity;
        self.last_refill = Instant::now();
        self.stats = RateLimiterStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_initial_capacity() {
        let mut bucket = TokenBucket::new(10.0, 2.0);
        assert!((bucket.tokens() - 10.0).abs() < 1e-4);
        assert_eq!(bucket.stats(), RateLimiterStats::default());
    }

    #[test]
    fn test_token_bucket_exhaustion() {
        let mut bucket = TokenBucket::new(2.0, 1.0);
        assert!(bucket.try_acquire(1.0));
        assert!(bucket.try_acquire(1.0));
        assert!(!bucket.try_acquire(1.0));
        let stats = bucket.stats();
        assert_eq!(stats.allowed, 2);
        assert_eq!(stats.dropped, 1);
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(5.0, 100.0);
        assert!(bucket.try_acquire(5.0));
        assert!(!bucket.try_acquire(1.0));
        std::thread::sleep(std::time::Duration::from_millis(30));
        assert!(bucket.try_acquire(1.0));
    }

    #[test]
    fn test_token_bucket_reset() {
        let mut bucket = TokenBucket::new(3.0, 1.0);
        assert!(bucket.try_acquire(3.0));
        assert!(!bucket.try_acquire(1.0));
        bucket.reset();
        assert_eq!(bucket.stats(), RateLimiterStats::default());
        assert!(bucket.try_acquire(3.0));
    }
}
