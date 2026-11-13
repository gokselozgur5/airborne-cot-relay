//! Resilience and fault-tolerance primitives for outbound CoT streaming.

pub mod breaker;

pub use breaker::{BreakerConfig, BreakerError, CircuitBreaker, CircuitState};
