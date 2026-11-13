use crate::clock::Clock;
use std::fmt;

/// State of a circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Error returned when an operation is rejected by an open circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakerError;

impl fmt::Display for BreakerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "circuit breaker is open")
    }
}

impl std::error::Error for BreakerError {}

/// Configuration tuning circuit breaker trip and recovery behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakerConfig {
    pub failure_threshold: u32,
    pub cooldown_ms: u64,
}

impl BreakerConfig {
    pub const fn new(failure_threshold: u32, cooldown_ms: u64) -> Self {
        Self {
            failure_threshold,
            cooldown_ms,
        }
    }
}

/// State machine tracking failures and enforcing cooldowns.
pub struct CircuitBreaker<C> {
    config: BreakerConfig,
    clock: C,
    consecutive_failures: u32,
    opened_at_ms: u64,
    rejected_count: u64,
    probe_in_flight: bool,
}
