//! Tests for SinkRecovery.
use airborne_cot_relay::relay::Backoff;
use airborne_cot_relay::runtime::reconnect::ReconnectLoop;
use airborne_cot_relay::runtime::sink_recovery::SinkRecovery;
use airborne_cot_relay::runtime::sleep::ManualSleeper;
use airborne_cot_relay::runtime::ShutdownSignal;

#[test]
fn initial_reconnects_is_zero() {
    let loop_inst = ReconnectLoop::new(
        Backoff::new(100, 1000),
        ManualSleeper::new(),
        ShutdownSignal::new(),
    );
    let recovery = SinkRecovery::new(loop_inst);
    assert_eq!(recovery.reconnects(), 0);
}

use airborne_cot_relay::RelayError;

#[test]
fn sink_degraded_triggers_reconnect_loop_and_counts_reconnects() {
    let reconnect_loop = ReconnectLoop::new(
        Backoff::new(100, 1000),
        ManualSleeper::new(),
        ShutdownSignal::new(),
    );
    let mut recovery = SinkRecovery::new(reconnect_loop);
    let mut calls = 0;
    let mut connect = || {
        calls += 1;
        if calls <= 2 {
            Err(RelayError::IoError("refused".into()))
        } else {
            Ok(42)
        }
    };
    let event = airborne_cot_relay::pipeline::events::PipelineEvent::SinkDegraded { failures: 3 };
    let res = recovery.on_event(&event, &mut connect);
    assert_eq!(res.unwrap(), Some(42));
    assert_eq!(recovery.reconnects(), 1);
    assert_eq!(calls, 3);
}

#[test]
fn non_degraded_events_do_not_reconnect() {
    let reconnect_loop = ReconnectLoop::new(
        Backoff::new(100, 1000),
        ManualSleeper::new(),
        ShutdownSignal::new(),
    );
    let mut recovery = SinkRecovery::new(reconnect_loop);
    let mut called = false;
    let mut connect = || {
        called = true;
        Ok(42)
    };
    let event =
        airborne_cot_relay::pipeline::events::PipelineEvent::TelemetryStale { age_ms: 5000 };
    let res: Result<Option<i32>, RelayError> = recovery.on_event(&event, &mut connect);
    assert_eq!(res.unwrap(), None);
    assert!(!called);
    assert_eq!(recovery.reconnects(), 0);
}

use airborne_cot_relay::relay::{BoundedQueue, DropPolicy};

#[test]
fn queue_preserves_events_within_drop_policy() {
    let reconnect_loop = ReconnectLoop::new(
        Backoff::new(100, 1000),
        ManualSleeper::new(),
        ShutdownSignal::new(),
    );
    let mut recovery = SinkRecovery::new(reconnect_loop);
    let mut queue = BoundedQueue::new(2, DropPolicy::DropOldest);
    queue.push(10);
    queue.push(20);
    queue.push(30);
    assert_eq!(queue.len(), 2);
    let event = airborne_cot_relay::pipeline::events::PipelineEvent::SinkDegraded { failures: 3 };
    let _ = recovery.on_event(&event, &mut || Ok(1));
    assert_eq!(queue.pop(), Some(20));
    assert_eq!(queue.pop(), Some(30));
}
