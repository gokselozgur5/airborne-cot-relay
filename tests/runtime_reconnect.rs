//! Tests for retrying a connect closure on a Backoff schedule.

use airborne_cot_relay::relay::Backoff;
use airborne_cot_relay::runtime::reconnect::ReconnectLoop;
use airborne_cot_relay::runtime::sleep::ManualSleeper;
use airborne_cot_relay::runtime::ShutdownSignal;
use airborne_cot_relay::RelayError;
use std::time::Duration;

fn refused() -> RelayError {
    RelayError::IoError("connection refused".into())
}

fn manual_loop(base_ms: u64, shutdown: &ShutdownSignal) -> ReconnectLoop<ManualSleeper<'static>> {
    ReconnectLoop::new(
        Backoff::new(base_ms, 1_000),
        ManualSleeper::new(),
        shutdown.clone(),
    )
}
