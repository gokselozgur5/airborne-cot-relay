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

#[test]
fn shutdown_returns_the_next_failure_without_sleeping() {
    let shutdown = ShutdownSignal::new();
    let mut reconnect = manual_loop(50, &shutdown);
    let mut calls = 0;
    let result: Result<(), RelayError> = reconnect.run(|| {
        calls += 1;
        if calls == 3 {
            shutdown.trigger();
        }
        Err(refused())
    });
    assert_eq!(result, Err(refused()));
    assert_eq!(calls, 3);
    assert_eq!(reconnect.sleeper().slept().len(), 2);
}

#[test]
fn invalid_config_is_returned_without_retrying() {
    let mut reconnect = manual_loop(50, &ShutdownSignal::new());
    let bad_port = || RelayError::InvalidConfig("bad port".into());
    let result: Result<(), RelayError> = reconnect.run(|| Err(bad_port()));
    assert_eq!(result, Err(bad_port()));
    assert!(reconnect.sleeper().slept().is_empty());
}
