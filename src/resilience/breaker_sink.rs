use crate::clock::Clock;
use crate::config::RelayError;
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

    pub fn rejected_count(&self) -> u64 {
        self.breaker.rejected_count()
    }

    pub fn state(&self) -> CircuitState {
        self.breaker.state()
    }
}

impl<S: CotSink, C: Clock> CotSink for BreakerSink<S, C> {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        if self.breaker.allow_request().is_err() {
            return Err(RelayError::IoError("circuit breaker is open".into()));
        }
        match self.inner.send(payload) {
            Ok(n) => {
                self.breaker.record_success();
                Ok(n)
            }
            Err(err) => {
                self.breaker.record_failure();
                Err(err)
            }
        }
    }
}
