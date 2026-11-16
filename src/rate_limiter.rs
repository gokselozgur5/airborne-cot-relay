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
