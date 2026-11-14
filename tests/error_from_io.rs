use airborne_cot_relay::RelayError;
use std::io::{self, ErrorKind};

#[test]
fn connection_refused_keeps_kind_name() {
    let err = RelayError::from(io::Error::from(ErrorKind::ConnectionRefused));
    assert!(matches!(err, RelayError::IoError(_)));
    assert!(err.to_string().contains("ConnectionRefused"), "{err}");
}

#[test]
fn custom_message_is_preserved() {
    let err = RelayError::from(io::Error::new(ErrorKind::AddrInUse, "port 6969"));
    let text = err.to_string();
    assert!(text.starts_with("IO error: AddrInUse"), "{text}");
    assert!(text.contains("port 6969"), "{text}");
}

#[test]
fn question_mark_converts_io_error() {
    fn bind() -> Result<(), RelayError> {
        Err(io::Error::from(ErrorKind::PermissionDenied))?;
        Ok(())
    }
    assert!(bind().unwrap_err().to_string().contains("PermissionDenied"));
}

#[test]
fn borrowed_io_error_keeps_kind_name() {
    let io_err = io::Error::new(ErrorKind::BrokenPipe, "pipe broken");
    let err = RelayError::from(&io_err);
    assert!(matches!(err, RelayError::IoError(_)));
    assert!(err.to_string().contains("BrokenPipe"), "{err}");
    assert!(err.to_string().contains("pipe broken"), "{err}");
}

#[test]
fn error_kind_preserves_name() {
    let err = RelayError::from(ErrorKind::TimedOut);
    assert!(matches!(err, RelayError::IoError(_)));
    assert!(err.to_string().contains("TimedOut"), "{err}");
}
