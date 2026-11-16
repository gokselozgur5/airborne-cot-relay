//! Dual-buffer lock-free telemetry snapshot sampler.

use crate::PlatformPose;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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

/// Sampler operational metrics and contention stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SamplerStats {
    pub writes_total: u64,
    pub reads_total: u64,
    pub retries_total: u64,
}

/// Lock-free dual-buffer telemetry snapshot sampler.
pub struct DualBufferSampler {
    buffers: [UnsafeCell<TelemetrySnapshot>; 2],
    versions: [AtomicU64; 2],
    active_idx: AtomicUsize,
    writes_total: AtomicU64,
    reads_total: AtomicU64,
    retries_total: AtomicU64,
}

unsafe impl Sync for DualBufferSampler {}
unsafe impl Send for DualBufferSampler {}
