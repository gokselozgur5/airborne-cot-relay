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
