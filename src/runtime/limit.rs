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
