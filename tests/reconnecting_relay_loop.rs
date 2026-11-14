use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::relay::{CotSink, DropPolicy, ManualClock};
use airborne_cot_relay::relay_loop::ReconnectingRelayLoop;
use std::sync::Arc;

struct MockSink;
impl CotSink for MockSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        Ok(payload.len())
    }
}

#[test]
fn test_reconnecting_relay_loop_type_exists() {
    let clock = ManualClock::new(0);
    let metrics = Arc::new(Metrics::new());
    let _ = ReconnectingRelayLoop::new(
        MockSink,
        || Ok(MockSink),
        10,
        DropPolicy::DropOldest,
        clock,
        metrics,
    );
}

#[test]
fn test_reconnecting_relay_loop_enqueue_and_drain() {
    let clock = ManualClock::new(100);
    let metrics = Arc::new(Metrics::new());
    let mut loop_inst = ReconnectingRelayLoop::new(
        MockSink,
        || Ok(MockSink),
        5,
        DropPolicy::DropOldest,
        clock,
        Arc::clone(&metrics),
    );
    loop_inst.enqueue(b"hello".to_vec());
    loop_inst.enqueue(b"world".to_vec());
    assert_eq!(metrics.snapshot(100).queue_depth, 2);
    loop_inst.poll().unwrap();
    let snap = metrics.snapshot(100);
    assert_eq!(snap.queue_depth, 0);
    assert_eq!(snap.sent, 2);
}
