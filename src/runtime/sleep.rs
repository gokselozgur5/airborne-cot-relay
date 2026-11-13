//! Injectable sleeping so loops can be tested without real delays.

use crate::relay::ManualClock;
use std::time::Duration;

/// Blocks the caller for a duration.
pub trait Sleeper {
    fn sleep(&mut self, duration: Duration);
}

/// Sleeper backed by `std::thread::sleep`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ThreadSleeper;

impl Sleeper for ThreadSleeper {
    fn sleep(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Sleeper that records requested durations and returns immediately.
#[derive(Debug, Default)]
pub struct ManualSleeper<'a> {
    slept: Vec<Duration>,
    clock: Option<&'a ManualClock>,
}

impl<'a> ManualSleeper<'a> {
    /// Creates a sleeper with no attached clock.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sleeper that advances `clock` by every requested duration.
    pub fn with_clock(clock: &'a ManualClock) -> Self {
        let slept = Vec::new();
        let clock = Some(clock);
        Self { slept, clock }
    }

    /// Durations requested so far, in call order.
    pub fn slept(&self) -> &[Duration] {
        &self.slept
    }
}

impl Sleeper for ManualSleeper<'_> {
    fn sleep(&mut self, duration: Duration) {
        self.slept.push(duration);
        if let Some(clock) = self.clock {
            clock.advance_ms(u64::try_from(duration.as_millis()).unwrap_or(u64::MAX));
        }
    }
}
