//! Injectable sleeping so loops can be tested without real delays.

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
