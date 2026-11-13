use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::RelayConfig;
use stm32_navcore::protocol::NavTelemetry;

fn frame(timestamp_ms: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms,
        pos_ned_m: [0.0, 0.0, -100.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).unwrap();
    buf[..len].to_vec()
}

fn pipeline() -> Pipeline {
    Pipeline::from_config(&RelayConfig::default()).unwrap()
}

#[test]
fn two_frames_yield_two_poses_and_two_packets() {
    let mut p = pipeline();
    let mut bytes = frame(1000);
    bytes.extend(frame(1100));
    let poses = p.ingest_telemetry(&bytes);
    assert_eq!(poses.len(), 2);
    let snap = p.metrics().snapshot(0);
    assert_eq!(snap.packets_in, 2);
    assert_eq!(snap.dropped, 0);
    assert_eq!(p.latest_pose().unwrap().timestamp_ms, 1100);
}

#[test]
fn corrupt_frame_increments_dropped_only() {
    let mut p = pipeline();
    let mut bytes = vec![0x05, 0x11, 0x22, 0x00];
    bytes.extend(frame(1000));
    assert_eq!(p.ingest_telemetry(&bytes).len(), 1);
    let snap = p.metrics().snapshot(0);
    assert_eq!(snap.packets_in, 1);
    assert_eq!(snap.dropped, 1);
}
