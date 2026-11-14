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
