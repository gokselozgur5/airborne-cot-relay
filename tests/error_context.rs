use airborne_cot_relay::error::context::ResultExt;
use airborne_cot_relay::RelayError;

#[test]
fn io_error_gets_prefixed_message() {
    let res: Result<(), RelayError> = Err(RelayError::IoError("x".into()));
    let err = res.context("bind telemetry").unwrap_err();
    assert_eq!(err, RelayError::IoError("bind telemetry: x".into()));
    assert_eq!(err.to_string(), "IO error: bind telemetry: x");
}

#[test]
fn ok_value_passes_through() {
    let res: Result<u8, RelayError> = Ok(7);
    assert_eq!(res.context("unused"), Ok(7));
}
