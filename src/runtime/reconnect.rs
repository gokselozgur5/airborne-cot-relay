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
