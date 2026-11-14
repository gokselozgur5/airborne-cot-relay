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
