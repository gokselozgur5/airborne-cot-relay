use airborne_cot_relay::clock::ManualClock;
use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::relay::CotSink;
use airborne_cot_relay::resilience::{BreakerConfig, BreakerSink, CircuitBreaker};

#[derive(Default)]
struct MockSink {
    calls: usize,
    fail_until: usize,
}

impl CotSink for MockSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.calls += 1;
        if self.calls <= self.fail_until {
            Err(RelayError::IoError("sink failed".into()))
        } else {
            Ok(payload.len())
        }
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

#[test]
fn test_open_breaker_rejects_without_calling_inner_sink() {
    let clock = ManualClock::new(100);
    let breaker = CircuitBreaker::new(BreakerConfig::new(2, 500), &clock);
    let inner = MockSink {
        calls: 0,
        fail_until: 2,
    };
    let mut sink = BreakerSink::new(inner, breaker);

    assert!(sink.send(b"1").is_err());
    assert!(sink.send(b"2").is_err());
    assert_eq!(sink.inner().calls, 2);
    assert!(sink.send(b"3").is_err());
    assert_eq!(sink.inner().calls, 2);
    assert_eq!(sink.rejected_count(), 1);

    clock.advance_ms(500);
    assert_eq!(sink.send(b"4").unwrap(), 1);
    assert_eq!(sink.inner().calls, 3);
}
