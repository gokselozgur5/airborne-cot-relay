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

#[test]
fn three_failures_then_success_sleep_first_three_delays() {
    let mut reconnect = manual_loop(100, &ShutdownSignal::new());
    let mut calls = 0;
    let result = reconnect.run(|| {
        calls += 1;
        (calls > 3).then_some(calls).ok_or_else(refused)
    });
    assert_eq!(result, Ok(4));
    let expected = [100, 200, 400].map(Duration::from_millis);
    assert_eq!(reconnect.sleeper().slept(), expected);
    assert_eq!(reconnect.backoff().attempt(), 0);
}
