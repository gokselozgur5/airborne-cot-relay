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

impl DualBufferSampler {
    /// Create a new sampler with an initial pose.
    pub fn new(initial_pose: PlatformPose, timestamp_ns: u64) -> Self {
        let snap = TelemetrySnapshot::new(initial_pose, 0, timestamp_ns);
        Self {
            buffers: [UnsafeCell::new(snap), UnsafeCell::new(snap)],
            versions: [AtomicU64::new(0), AtomicU64::new(0)],
            active_idx: AtomicUsize::new(0),
            writes_total: AtomicU64::new(0),
            reads_total: AtomicU64::new(0),
            retries_total: AtomicU64::new(0),
        }
    }

    /// Update the telemetry pose via lock-free back-buffer publication.
    pub fn update(&self, pose: PlatformPose, timestamp_ns: u64) -> u64 {
        let read_idx = self.active_idx.load(Ordering::Acquire);
        let write_idx = 1 - read_idx;
        let v = self.versions[write_idx].load(Ordering::Relaxed);
        self.versions[write_idx].store(v.wrapping_add(1), Ordering::Release);
        let seq = self.writes_total.fetch_add(1, Ordering::Relaxed) + 1;
        unsafe {
            *self.buffers[write_idx].get() = TelemetrySnapshot::new(pose, seq, timestamp_ns);
        }
        self.versions[write_idx].store(v.wrapping_add(2), Ordering::Release);
        self.active_idx.store(write_idx, Ordering::Release);
        seq
    }
}
