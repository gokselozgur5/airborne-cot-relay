//! Resilience and fault-tolerance primitives for outbound CoT streaming.

pub mod breaker;
pub mod breaker_sink;

pub use breaker::{BreakerConfig, BreakerError, CircuitBreaker, CircuitState};
pub use breaker_sink::BreakerSink;
