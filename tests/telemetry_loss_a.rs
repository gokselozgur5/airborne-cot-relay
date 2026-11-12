use airborne_cot_relay::TelemetryIngest;
use stm32_navcore::protocol::NavTelemetry;

fn frame(ts: u32, north: f32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms: ts,
        pos_ned_m: [north, 0.0, 0.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).unwrap();
    buf[..len].to_vec()
}

#[test]
fn older_frame_does_not_replace_latest_pose() {
    let mut ingest = TelemetryIngest::new(35.0, 139.0, 200.0, 5000);
    ingest.process_bytes(&frame(2000, 10.0));
    ingest.process_bytes(&frame(1000, 99.0));
    assert_eq!(ingest.latest_pose().unwrap().timestamp_ms, 2000);
}
