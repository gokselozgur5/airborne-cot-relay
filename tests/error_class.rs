use airborne_cot_relay::error::class::{classify, ErrorClass};
use airborne_cot_relay::RelayError;

#[test]
fn invalid_config_is_fatal() {
    let class = classify(&RelayError::InvalidConfig("bad addr".into()));
    assert_eq!(class, ErrorClass::Fatal);
    assert!(class.is_fatal() && !class.is_retryable());
}

#[test]
fn io_error_is_retryable() {
    let class = classify(&RelayError::IoError("refused".into()));
    assert_eq!(class, ErrorClass::Retryable);
    assert!(class.is_retryable() && !class.is_fatal());
}
