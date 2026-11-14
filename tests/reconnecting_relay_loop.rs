use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::relay::{CotSink, ManualClock, ReconnectingRelayLoop};

struct MockSink;
impl CotSink for MockSink {
    fn send(&mut self, _payload: &[u8]) -> Result<usize, RelayError> {
        Ok(0)
    }
}

#[test]
fn test_reconnecting_relay_loop_type_exists() {
    let clock = ManualClock::new(0);
    let _ = ReconnectingRelayLoop::new(MockSink, || Ok(MockSink), 10, clock);
}
