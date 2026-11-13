//! Clock wrapper that never steps backward.

use std::sync::atomic::{AtomicU64, Ordering};

/// Wraps a clock and holds the last value when the inner clock steps back.
#[derive(Debug, Default)]
pub struct MonotonicClock<C> {
    inner: C,
    last_ms: AtomicU64,
    backward_steps: AtomicU64,
}

impl<C> MonotonicClock<C> {
    /// Wraps `inner`; the first read is returned unchanged.
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            last_ms: AtomicU64::new(0),
            backward_steps: AtomicU64::new(0),
        }
    }
}
