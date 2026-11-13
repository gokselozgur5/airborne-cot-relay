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
}
