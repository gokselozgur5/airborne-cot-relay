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

impl<C: Clock> CircuitBreaker<C> {
    pub fn new(config: BreakerConfig, clock: C) -> Self {
        Self {
            config,
            clock,
            consecutive_failures: 0,
            opened_at_ms: 0,
            rejected_count: 0,
            probe_in_flight: false,
        }
    }

    pub fn rejected_count(&self) -> u64 {
        self.rejected_count
    }

    pub fn state(&self) -> CircuitState {
        if self.consecutive_failures < self.config.failure_threshold {
            CircuitState::Closed
        } else if self.clock.now_ms().saturating_sub(self.opened_at_ms) >= self.config.cooldown_ms {
            CircuitState::HalfOpen
        } else {
            CircuitState::Open
        }
    }

    pub fn allow_request(&mut self) -> Result<(), BreakerError> {
        match self.state() {
            CircuitState::Closed => Ok(()),
            CircuitState::HalfOpen if !self.probe_in_flight => {
                self.probe_in_flight = true;
                Ok(())
            }
            _ => {
                self.rejected_count += 1;
                Err(BreakerError)
            }
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.probe_in_flight = false;
    }

    pub fn record_failure(&mut self) {
        if self.consecutive_failures < self.config.failure_threshold {
            self.consecutive_failures += 1;
        }
        self.opened_at_ms = self.clock.now_ms();
        self.probe_in_flight = false;
    }
}
