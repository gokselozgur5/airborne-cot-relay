//! Bounds on how long a loop may run.

use crate::relay::Clock;

/// Stops a loop after a number of iterations or at a clock deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunLimit {
    /// Runs until stopped by other means.
    Unbounded,
    /// Runs this many more iterations.
    Iterations(u64),
    /// Runs while the clock reads earlier than this time in milliseconds.
    Deadline(i64),
}

impl RunLimit {
    /// Limit that never stops the loop.
    pub fn unbounded() -> Self {
        Self::Unbounded
    }

    /// Limit that allows exactly `count` iterations.
    pub fn iterations(count: u64) -> Self {
        Self::Iterations(count)
    }

    /// Limit that stops once the clock reaches `deadline_ms`.
    pub fn deadline_ms(deadline_ms: i64) -> Self {
        Self::Deadline(deadline_ms)
    }

    /// Limit that stops `duration_ms` after the current time of `clock`.
    pub fn deadline_after(clock: &impl Clock, duration_ms: u64) -> Self {
        let duration = i64::try_from(duration_ms).unwrap_or(i64::MAX);
        Self::Deadline(clock.now_ms().saturating_add(duration))
    }
}
