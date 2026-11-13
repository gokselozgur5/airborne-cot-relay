use airborne_cot_relay::RelayError;
use sightline_cot::cot::CotError;

#[test]
fn network_error_maps_to_io_error() {
    let err = RelayError::from(CotError::NetworkError("reset by peer".into()));
    assert_eq!(
        err,
        RelayError::IoError("network error: reset by peer".into())
    );
}

#[test]
fn invalid_xml_maps_to_decode_error() {
    let err = RelayError::from(CotError::InvalidXml("unclosed tag".into()));
    assert_eq!(
        err,
        RelayError::DecodeError("invalid XML: unclosed tag".into())
    );
}
