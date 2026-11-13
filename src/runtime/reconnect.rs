//! Retrying a connect closure on a `Backoff` schedule.

use super::sleep::Sleeper;
use super::ShutdownSignal;
use crate::config::RelayError;
use crate::relay::Backoff;
use std::time::Duration;

/// Retries a connect closure, sleeping the next `Backoff` delay after each failure.
#[derive(Debug)]
pub struct ReconnectLoop<S: Sleeper> {
    backoff: Backoff,
    sleeper: S,
    shutdown: ShutdownSignal,
}

impl<S: Sleeper> ReconnectLoop<S> {
    /// Creates a loop that stops retrying once `shutdown` is triggered.
    pub fn new(backoff: Backoff, sleeper: S, shutdown: ShutdownSignal) -> Self {
        Self {
            backoff,
            sleeper,
            shutdown,
        }
    }

    /// The backoff schedule, for inspecting the attempt counter.
    pub fn backoff(&self) -> &Backoff {
        &self.backoff
    }

    /// The sleeper, for inspecting recorded delays.
    pub fn sleeper(&self) -> &S {
        &self.sleeper
    }

    /// Calls `connect` until it succeeds or shutdown is requested.
    ///
    /// Success resets the backoff; after shutdown the last error is returned.
    pub fn run<T, F>(&mut self, mut connect: F) -> Result<T, RelayError>
    where
        F: FnMut() -> Result<T, RelayError>,
    {
        loop {
            match connect() {
                Ok(value) => {
                    self.backoff.reset();
                    return Ok(value);
                }
                Err(err) if self.shutdown.is_triggered() => return Err(err),
                Err(_) => {
                    let delay = self.backoff.next_delay_ms();
                    self.sleeper.sleep(Duration::from_millis(delay));
                }
            }
        }
    }
}
