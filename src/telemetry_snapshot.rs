//! Dual-buffer lock-free telemetry snapshot sampler.

use crate::PlatformPose;

/// Snapshot of platform telemetry at a specific sequence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySnapshot {
    pub pose: PlatformPose,
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub is_valid: bool,
}

pub struct SamplerStats;
pub struct DualBufferSampler;
