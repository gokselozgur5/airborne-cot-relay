use airborne_cot_relay::TelemetryIngest;
use stm32_navcore::protocol::NavTelemetry;

fn frame(ts: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms: ts,
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
fn buffer_is_capped_without_delimiter() {
    let mut ingest = TelemetryIngest::new(35.0, 139.0, 200.0, 5000);
    for _ in 0..4 {
        ingest.process_bytes(&vec![0x55u8; 1 << 20]);
    }
    assert!(ingest.buffered_len() <= 128);
    assert!(ingest.dropped_bytes() > 0);
    ingest.process_bytes(&[0x00]);
    assert_eq!(ingest.process_bytes(&frame(1000)).len(), 1);
}
