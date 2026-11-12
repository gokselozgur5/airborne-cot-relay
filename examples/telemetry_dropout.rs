//! Telemetry dropout walkthrough.
//!
//! Encodes navigation frames, feeds them to the telemetry ingest and reports
//! when the latest platform pose becomes stale during a link dropout.

use airborne_cot_relay::TelemetryIngest;
use stm32_navcore::protocol::NavTelemetry;

/// Encode one navigation frame with the given timestamp as UART bytes.
fn encode_frame(timestamp_ms: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms,
        pos_ned_m: [100.0, -50.0, -30.0],
        vel_ned_mps: [15.0, 0.0, 0.0],
        euler_rad: [0.05, -0.02, 0.8],
        status: 1,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).expect("frame fits in buffer");
    buf[..len].to_vec()
}

fn main() {
    // Poses older than this are reported as stale.
    let max_age_ms = 1_500;
    let mut ingest = TelemetryIngest::new(37.7749, -122.4194, 100.0, max_age_ms);
    // The link delivers frames until 2 s, drops out for 3 s, then recovers.
    let frame_times = [0u64, 1_000, 2_000, 5_000];
    for now_ms in (0..=5_000u64).step_by(1_000) {
        if frame_times.contains(&now_ms) {
            let poses = ingest.process_bytes(&encode_frame(now_ms as u32));
            println!("t={now_ms:>5} ms: decoded {} pose(s)", poses.len());
        }
        println!("t={now_ms:>5} ms: stale={}", ingest.is_stale(now_ms));
    }
}
