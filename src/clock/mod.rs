//! Unified millisecond clock shared by every relay component.
//!
//! Time is expressed as milliseconds since the Unix epoch in a `u64`.

pub mod adapters;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

impl<C: Clock + ?Sized> Clock for Box<C> {
    fn now_ms(&self) -> u64 {
        (**self).now_ms()
    }
}

/// Monotonic clock anchored once at process start to Unix epoch milliseconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    /// Creates a new monotonic system clock handle.
    pub fn new() -> Self {
        Self
    }
}

fn monotonic_epoch_ms() -> u64 {
    static ANCHOR: OnceLock<(u64, Instant)> = OnceLock::new();
    let &(base_ms, start) = ANCHOR.get_or_init(|| {
        let now = Instant::now();
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        (wall, now)
    });
    let elapsed = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    base_ms.saturating_add(elapsed)
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        monotonic_epoch_ms()
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

    /// Returns the current manual clock reading in milliseconds.
    pub fn now_ms(&self) -> u64 {
        self.now_ms.load(Ordering::SeqCst)
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        ManualClock::now_ms(self)
    }
}

impl ManualClock {
    /// Moves the clock forward by `delta_ms`, saturating at `u64::MAX`.
    pub fn advance_ms(&self, delta_ms: u64) {
        let mut cur = self.now_ms.load(Ordering::SeqCst);
        let next = |t: u64| t.saturating_add(delta_ms);
        while let Err(seen) =
            self.now_ms
                .compare_exchange_weak(cur, next(cur), Ordering::SeqCst, Ordering::SeqCst)
        {
            cur = seen;
        }
    }

    /// Alias for [`ManualClock::advance_ms`].
    pub fn advance(&self, delta_ms: u64) {
        self.advance_ms(delta_ms);
    }

    /// Sets the clock to `now_ms`; backward steps are allowed.
    pub fn set_ms(&self, now_ms: u64) {
        self.now_ms.store(now_ms, Ordering::SeqCst);
    }

    /// Alias for [`ManualClock::set_ms`].
    pub fn set(&self, now_ms: u64) {
        self.set_ms(now_ms);
    }
}

pub mod monotonic;
