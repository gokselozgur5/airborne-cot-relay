use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::error::context::ResultExt;

#[test]
fn test_context_prefixes_relay_error_message() {
    let err: Result<(), RelayError> = Err(RelayError::IoError("connection refused".into()));
    let with_ctx = err.context("bind telemetry");
    assert_eq!(
        with_ctx.unwrap_err().to_string(),
        "IO error: bind telemetry: connection refused"
    );
}

#[test]
fn test_context_preserves_ok_value() {
    let ok: Result<u32, RelayError> = Ok(42);
    assert_eq!(ok.context("should not prefix"), Ok(42));
}
