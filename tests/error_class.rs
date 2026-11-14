use std::io::{self, ErrorKind};

use crate::error::{ErrorClass, From};

#[test]
fn test_error_class() {
    assert_eq!(ErrorClass::is_fatal(&ErrorClass::Fatal), true);
    assert_eq!(ErrorClass::is_retryable(&ErrorClass::Retryable), true);
    assert_eq!(ErrorClass::is_retryable(&ErrorClass::Droppable), false);

    let io_error = io::Error::new(ErrorKind::ConnectionRefused, "connection refused");
    assert_eq!(ErrorClass::from(io_error), ErrorClass::Retryable);

    let io_timeout = io::Error::new(ErrorKind::TimedOut, "timeout");
    assert_eq!(ErrorClass::from(io_timeout), ErrorClass::Fatal);

    let decode_error = io::Error::new(ErrorKind::InvalidData, "invalid data");
    assert_eq!(ErrorClass::from(decode_error), ErrorClass::Droppable);

    let targeting_error = io::Error::new(ErrorKind::Other, "other error");
    assert_eq!(ErrorClass::from(targeting_error), ErrorClass::Fatal);
}
