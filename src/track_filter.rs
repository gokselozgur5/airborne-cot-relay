//! Deduplication and rate limiting for outbound track events.

use std::cell::Cell;
use std::collections::HashMap;

/// Source of monotonic time in milliseconds.
pub trait Clock {
    fn now_ms(&self) -> u64;
}

/// Manually advanced clock for deterministic tests.
#[derive(Debug, Default)]
pub struct ManualClock(Cell<u64>);

impl ManualClock {
    pub fn new(start_ms: u64) -> Self {
        Self(Cell::new(start_ms))
    }

    pub fn advance(&self, delta_ms: u64) {
        self.0.set(self.0.get() + delta_ms);
    }
}
