//! Unified millisecond clock shared by every relay component.
//!
//! Time is expressed as milliseconds since the Unix epoch in a `u64`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Source of time in milliseconds.
pub trait Clock {
    /// Returns the current time in milliseconds.
    fn now_ms(&self) -> u64;
}

impl<C: Clock + ?Sized> Clock for &C {
    fn now_ms(&self) -> u64 {
        (**self).now_ms()
    }
}

impl<C: Clock + ?Sized> Clock for Arc<C> {
    fn now_ms(&self) -> u64 {
        (**self).now_ms()
    }
}

/// Wall clock that never returns less than a value it already returned.
#[derive(Debug, Default)]
pub struct SystemClock {
    last_ms: AtomicU64,
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let prev = self.last_ms.fetch_max(wall, Ordering::Relaxed);
        prev.max(wall)
    }
}

/// Deterministic, thread-safe clock that only moves when told to.
#[derive(Debug, Default)]
pub struct ManualClock {
    now_ms: AtomicU64,
}

impl ManualClock {
    /// Creates a clock frozen at `start_ms`.
    pub fn new(start_ms: u64) -> Self {
        Self {
            now_ms: AtomicU64::new(start_ms),
        }
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.load(Ordering::SeqCst)
    }
}
