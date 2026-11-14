//! Adapters exposing a unified clock through older clock traits.

use super::Clock;

/// Exposes a unified clock as a `relay::Clock` in signed milliseconds.
///
/// Values above `i64::MAX` saturate instead of wrapping.
#[derive(Debug, Default, Clone, Copy)]
pub struct RelayClockAdapter<C>(pub C);

impl<C> RelayClockAdapter<C> {
    /// Creates a new adapter wrapping `clock`.
    pub fn new(clock: C) -> Self {
        Self(clock)
    }

    /// Returns a reference to the wrapped clock.
    pub fn inner(&self) -> &C {
        &self.0
    }

    /// Consumes the adapter, returning the wrapped clock.
    pub fn into_inner(self) -> C {
        self.0
    }
}

impl<C> From<C> for RelayClockAdapter<C> {
    fn from(clock: C) -> Self {
        Self(clock)
    }
}

impl<C: Clock> crate::relay::Clock for RelayClockAdapter<C> {
    fn now_ms(&self) -> i64 {
        i64::try_from(self.0.now_ms()).unwrap_or(i64::MAX)
    }
}

/// Exposes a unified clock as a `health::TimeSource`.
#[derive(Debug, Default, Clone, Copy)]
pub struct HealthTimeAdapter<C>(pub C);

impl<C> HealthTimeAdapter<C> {
    /// Wraps a clock in a health time adapter.
    pub fn new(clock: C) -> Self {
        Self(clock)
    }

    /// Borrows the underlying clock.
    pub fn inner(&self) -> &C {
        &self.0
    }

    /// Consumes the adapter, returning the underlying clock.
    pub fn into_inner(self) -> C {
        self.0
    }
}

impl<C: Clock> crate::health::TimeSource for HealthTimeAdapter<C> {
    fn now(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.0.now_ms())
    }
}

impl<C: Clock> Clock for HealthTimeAdapter<C> {
    fn now_ms(&self) -> u64 {
        self.0.now_ms()
    }
}
