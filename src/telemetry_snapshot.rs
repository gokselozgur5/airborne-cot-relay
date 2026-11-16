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

impl TelemetrySnapshot {
    /// Create a new snapshot instance.
    pub const fn new(
        pose: PlatformPose,
        sequence: u64,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            pose,
            sequence,
            timestamp_ns,
            is_valid: true,
        }
    }
}

pub struct SamplerStats;
pub struct DualBufferSampler;
