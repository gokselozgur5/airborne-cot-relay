use airborne_cot_relay::{RelayError, TelemetryIngest};
use stm32_navcore::protocol::NavTelemetry;

fn good_frame() -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms: 1000,
        pos_ned_m: [1.0, 0.0, 0.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).unwrap();
    buf[..len].to_vec()
}

#[test]
fn garbage_frame_is_counted_and_recorded() {
    let mut ingest = TelemetryIngest::new(35.0, 139.0, 200.0, 5000);
    assert!(ingest.process_bytes(&[0x05, 0x11, 0x22, 0x00]).is_empty());
    assert_eq!(ingest.decode_errors(), 1);
    assert!(matches!(
        ingest.last_decode_error(),
        Some(RelayError::DecodeError(_))
    ));
}
