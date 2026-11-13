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
