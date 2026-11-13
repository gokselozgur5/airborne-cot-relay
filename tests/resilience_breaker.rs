use airborne_cot_relay::clock::ManualClock;
use airborne_cot_relay::resilience::{BreakerConfig, CircuitBreaker, CircuitState};

#[test]
fn test_circuit_breaker_transitions() {
    let clock = ManualClock::new(1000);
    let mut cb = CircuitBreaker::new(BreakerConfig::new(2, 500), &clock);
    assert!(cb.allow_request().is_ok());
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(cb.allow_request().is_err());
    assert_eq!(cb.rejected_count(), 1);
    clock.advance_ms(499);
    assert_eq!(cb.state(), CircuitState::Open);
    clock.advance_ms(1);
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    assert!(cb.allow_request().is_ok());
    assert!(cb.allow_request().is_err());
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    clock.advance_ms(500);
    assert!(cb.allow_request().is_ok());
    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
}
