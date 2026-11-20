use airborne_cot_relay::clock::{Clock, SystemClock};
use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::{RelayConfig, TargetDetection};
use stm32_navcore::protocol::NavTelemetry;

fn frame(ts: u32) -> Vec<u8> {
    let t = NavTelemetry { timestamp_ms: ts, pos_ned_m: [0.0, 0.0, -100.0], vel_ned_mps: [0.0; 3], euler_rad: [0.0; 3], status: 0 };
    let mut b = [0u8; NavTelemetry::MAX_UART_FRAME_LEN];
    let n = t.pack_uart(&mut b).unwrap();
    b[..n].to_vec()
}

#[test]
fn pose_decoded_just_now_is_fresh_on_the_system_clock() {
    let mut p = Pipeline::from_config(&RelayConfig::default()).unwrap();
    assert_eq!(p.ingest_telemetry(&frame(60_000)).len(), 1);
    let det = TargetDetection { pixel_u: 640.0, pixel_v: 360.0, bbox: (630.0, 350.0, 650.0, 370.0), classification: "vehicle".into(), confidence: 0.9, embedding: None };
    let now = SystemClock::new().now_ms();
    let tracks = p.process_detections(&[det], now).unwrap();
    let dropped = p.metrics().snapshot(now).dropped;
    assert_eq!((tracks.len(), dropped), (1, 0));
}
