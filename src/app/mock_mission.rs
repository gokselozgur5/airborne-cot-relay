//! Deterministic mock mission: encoded telemetry frames plus detections.
use crate::targeter::TargetDetection;
use stm32_navcore::protocol::NavTelemetry;

/// Mission length in seconds.
pub const MOCK_DURATION_S: u32 = 60;
/// Telemetry and detection rate in hertz.
pub const MOCK_RATE_HZ: u32 = 10;
/// Timestamp of the first tick in milliseconds.
pub const MOCK_START_MS: u32 = 1_000;
/// Platform height above the telemetry origin in metres.
const ALTITUDE_M: f32 = 100.0;

/// One mission tick: an encoded telemetry frame and the detections seen at it.
#[derive(Debug, Clone, PartialEq)]
pub struct MockStep {
    pub timestamp_ms: u64,
    pub frame: Vec<u8>,
    pub detections: Vec<TargetDetection>,
}

/// Encodes a hovering, level platform frame as a COBS UART frame.
fn encode_frame(timestamp_ms: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms,
        pos_ned_m: [0.0, 0.0, -ALTITUDE_M],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; NavTelemetry::MAX_UART_FRAME_LEN];
    let len = telem
        .pack_uart(&mut buf)
        .expect("finite mock frame fits the UART buffer");
    buf[..len].to_vec()
}

/// Detection of the single mock vehicle, drifting one pixel per tick.
fn detection_at(tick: u32) -> TargetDetection {
    let u = 340.0 + f64::from(tick);
    let v = 360.0;
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 12.0, v - 8.0, u + 12.0, v + 8.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

/// Generates the full mission; every call returns the same sequence.
pub fn generate() -> Vec<MockStep> {
    let period_ms = 1000 / MOCK_RATE_HZ;
    (0..MOCK_DURATION_S * MOCK_RATE_HZ)
        .map(|tick| {
            let ts = MOCK_START_MS + tick * period_ms;
            MockStep {
                timestamp_ms: u64::from(ts),
                frame: encode_frame(ts),
                detections: vec![detection_at(tick)],
            }
        })
        .collect()
}
