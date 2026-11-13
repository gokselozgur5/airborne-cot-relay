use airborne_cot_relay::RelayError;
use sightline_cot::mgrs::GridError;
use sightline_cot::proto::ProtoError;
use stm32_navcore::protocol::ProtocolError;

fn decode_text(err: RelayError) -> String {
    match err {
        RelayError::DecodeError(msg) => msg,
        other => panic!("expected DecodeError, got {other:?}"),
    }
}

#[test]
fn proto_error_keeps_variant_payload() {
    let msg = decode_text(ProtoError::UnsupportedWireType(7).into());
    assert_eq!(msg, "TAK protobuf: UnsupportedWireType(7)");
}

#[test]
fn grid_error_keeps_variant_name() {
    let msg = decode_text(GridError::InvalidZone.into());
    assert_eq!(msg, "MGRS grid: InvalidZone");
}

#[test]
fn navcore_protocol_error_keeps_variant_name() {
    let msg = decode_text(ProtocolError::CrcMismatch.into());
    assert_eq!(msg, "navcore protocol: CrcMismatch");
}

#[test]
fn question_mark_converts_codec_errors() {
    fn decode(bad_crc: bool) -> Result<(), RelayError> {
        if bad_crc {
            Err(ProtocolError::CrcMismatch)?;
        }
        Err(ProtoError::InvalidMagic)?
    }
    assert!(decode(true)
        .unwrap_err()
        .to_string()
        .contains("CrcMismatch"));
    assert!(decode(false)
        .unwrap_err()
        .to_string()
        .contains("InvalidMagic"));
}
