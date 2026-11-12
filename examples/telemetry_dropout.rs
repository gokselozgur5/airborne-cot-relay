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
