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

#[test]
fn telemetry_ingest_accessor_provides_ingest_state() {
    let p = pipeline();
    assert_eq!(p.telemetry_ingest().buffered_len(), 0);
    assert_eq!(p.telemetry_ingest().decode_errors(), 0);
}

#[test]
fn telemetry_ingest_mut_and_clear_buffer() {
    let mut p = pipeline();
    p.telemetry_ingest_mut().clear_buffer();
    assert_eq!(p.telemetry_ingest().buffered_len(), 0);
    p.clear_telemetry_buffer();
    assert_eq!(p.telemetry_ingest().buffered_len(), 0);
}

#[test]
fn split_frame_across_chunks_updates_metrics_on_completion() {
    let mut p = pipeline();
    let bytes = frame(1000);
    let (chunk1, chunk2) = bytes.split_at(bytes.len() / 2);
    let poses1 = p.ingest_telemetry(chunk1);
    assert!(poses1.is_empty());
    assert_eq!(p.metrics().snapshot(0).packets_in, 0);

    let poses2 = p.ingest_telemetry(chunk2);
    assert_eq!(poses2.len(), 1);
    assert_eq!(p.metrics().snapshot(0).packets_in, 1);
    assert_eq!(p.metrics().snapshot(0).dropped, 0);
}

#[test]
fn stale_and_duplicate_frames_do_not_increment_packets_in() {
    let mut p = pipeline();
    assert_eq!(p.ingest_telemetry(&frame(2000)).len(), 1);
    assert!(p.ingest_telemetry(&frame(1000)).is_empty());
    assert!(p.ingest_telemetry(&frame(2000)).is_empty());
    assert_eq!(p.metrics().snapshot(0).packets_in, 1);
    assert_eq!(p.latest_pose().unwrap().timestamp_ms, 2000);
}
