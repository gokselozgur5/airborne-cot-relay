use crate::clock::Clock;
use crate::relay::CotSink;
use crate::resilience::breaker::{CircuitBreaker, CircuitState};

/// Destination sink guarded by a [`CircuitBreaker`].
pub struct BreakerSink<S, C> {
    inner: S,
    breaker: CircuitBreaker<C>,
}

impl<S: CotSink, C: Clock> BreakerSink<S, C> {
    pub fn new(inner: S, breaker: CircuitBreaker<C>) -> Self {
        Self { inner, breaker }
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn breaker(&self) -> &CircuitBreaker<C> {
        &self.breaker
    }

    pub fn state(&self) -> CircuitState {
        self.breaker.state()
    }
}
