use airborne_cot_relay::TelemetryIngest;
use stm32_navcore::protocol::NavTelemetry;

fn frame(ts: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms: ts,
        pos_ned_m: [1.0, 2.0, 3.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).unwrap();
    buf[..len].to_vec()
}

fn ingest() -> TelemetryIngest {
    TelemetryIngest::new(35.0, 139.0, 200.0, 5000)
}

#[test]
fn garbage_between_frames_is_skipped() {
    let mut stream = frame(1000);
    stream.extend_from_slice(&[0x11, 0x22, 0x33, 0x00]);
    stream.extend_from_slice(&frame(2000));
    let poses = ingest().process_bytes(&stream);
    let ts: Vec<u64> = poses.iter().map(|p| p.timestamp_ms).collect();
    assert_eq!(ts, vec![1000, 2000]);
}
