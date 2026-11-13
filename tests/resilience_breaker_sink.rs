use airborne_cot_relay::clock::ManualClock;
use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::relay::CotSink;
use airborne_cot_relay::resilience::{BreakerConfig, BreakerSink, CircuitBreaker};

#[derive(Default)]
struct MockSink {
    calls: usize,
}

impl CotSink for MockSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.calls += 1;
        Ok(payload.len())
    }
}

#[test]
fn test_successful_sends_pass_through() {
    let clock = ManualClock::new(100);
    let breaker = CircuitBreaker::new(BreakerConfig::new(2, 500), &clock);
    let mut sink = BreakerSink::new(MockSink::default(), breaker);
    assert_eq!(sink.send(b"hello").unwrap(), 5);
    assert_eq!(sink.inner().calls, 1);
}
