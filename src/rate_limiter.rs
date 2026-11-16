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
