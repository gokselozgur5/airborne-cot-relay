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

#[test]
fn every_variant_is_kept() {
    let cases = [
        RelayError::InvalidConfig("m".into()),
        RelayError::DecodeError("m".into()),
        RelayError::TargetingError("m".into()),
    ];
    for err in cases {
        let kind = std::mem::discriminant(&err);
        let out = Err::<(), _>(err).context("step").unwrap_err();
        assert_eq!(std::mem::discriminant(&out), kind);
        assert!(out.to_string().ends_with("step: m"), "{out}");
    }
}
