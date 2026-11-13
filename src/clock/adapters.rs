//! Adapters exposing a unified clock through older clock traits.

use super::Clock;

/// Exposes a unified clock as a `relay::Clock` in signed milliseconds.
///
/// Values above `i64::MAX` saturate instead of wrapping.
#[derive(Debug, Default, Clone, Copy)]
pub struct RelayClockAdapter<C>(pub C);

impl<C: Clock> crate::relay::Clock for RelayClockAdapter<C> {
    fn now_ms(&self) -> i64 {
        i64::try_from(self.0.now_ms()).unwrap_or(i64::MAX)
    }
}
