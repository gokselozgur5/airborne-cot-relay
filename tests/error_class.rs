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

#[test]
fn decode_and_targeting_errors_are_droppable() {
    for err in [
        RelayError::DecodeError("crc".into()),
        RelayError::TargetingError("no ground hit".into()),
    ] {
        let class = classify(&err);
        assert_eq!(class, ErrorClass::Droppable);
        assert!(!class.is_fatal() && !class.is_retryable());
    }
}

#[test]
fn converted_io_error_is_retryable() {
    let err = RelayError::from(std::io::Error::from(std::io::ErrorKind::TimedOut));
    assert_eq!(classify(&err), ErrorClass::Retryable);
}

#[test]
fn is_droppable_helper_returns_true_only_for_droppable() {
    assert!(ErrorClass::Droppable.is_droppable());
    assert!(!ErrorClass::Fatal.is_droppable());
    assert!(!ErrorClass::Retryable.is_droppable());
}

#[test]
fn as_str_returns_canonical_classifier_label() {
    assert_eq!(ErrorClass::Fatal.as_str(), "fatal");
    assert_eq!(ErrorClass::Retryable.as_str(), "retryable");
    assert_eq!(ErrorClass::Droppable.as_str(), "droppable");
}
