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
