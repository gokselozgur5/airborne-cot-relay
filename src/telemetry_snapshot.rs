//! Dual-buffer lock-free telemetry snapshot sampler.

use crate::PlatformPose;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

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
    pub const fn new(pose: PlatformPose, sequence: u64, timestamp_ns: u64) -> Self {
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
    snapshot: Mutex<TelemetrySnapshot>,
    active_idx: AtomicUsize,
    writes_total: AtomicU64,
    reads_total: AtomicU64,
    retries_total: AtomicU64,
}

impl DualBufferSampler {
    /// Create a new sampler with an initial pose.
    pub fn new(initial_pose: PlatformPose, timestamp_ns: u64) -> Self {
        let snap = TelemetrySnapshot::new(initial_pose, 0, timestamp_ns);
        Self {
            snapshot: Mutex::new(snap),
            active_idx: AtomicUsize::new(0),
            writes_total: AtomicU64::new(0),
            reads_total: AtomicU64::new(0),
            retries_total: AtomicU64::new(0),
        }
    }

    /// Update the telemetry pose under mutex synchronization.
    pub fn update(&self, pose: PlatformPose, timestamp_ns: u64) -> u64 {
        let seq = self.writes_total.fetch_add(1, Ordering::Relaxed) + 1;
        let snap = TelemetrySnapshot::new(pose, seq, timestamp_ns);
        let mut guard = self
            .snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = snap;
        let next_idx = 1 - self.active_idx.load(Ordering::Relaxed);
        self.active_idx.store(next_idx, Ordering::Relaxed);
        seq
    }

    /// Sample the latest consistent telemetry snapshot.
    pub fn sample(&self) -> TelemetrySnapshot {
        self.reads_total.fetch_add(1, Ordering::Relaxed);
        *self
            .snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Return current operational statistics.
    pub fn stats(&self) -> SamplerStats {
        SamplerStats {
            writes_total: self.writes_total.load(Ordering::Relaxed),
            reads_total: self.reads_total.load(Ordering::Relaxed),
            retries_total: self.retries_total.load(Ordering::Relaxed),
        }
    }

    /// Return current active read buffer index.
    pub fn active_index(&self) -> usize {
        self.active_idx.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_pose_and_sample() {
        let pose = PlatformPose::new(1000, 36.5, 32.2, 1500.0, 0.0, 0.0, 1.57);
        let sampler = DualBufferSampler::new(pose, 1_000_000);
        let snap = sampler.sample();
        assert_eq!(snap.pose, pose);
        assert_eq!(snap.sequence, 0);
        assert!(snap.is_valid);
    }

    #[test]
    fn test_buffer_alternation_and_update() {
        let p0 = PlatformPose::new(1000, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0);
        let sampler = DualBufferSampler::new(p0, 1000);
        assert_eq!(sampler.active_index(), 0);

        let p1 = PlatformPose::new(1010, 1.0, 1.0, 110.0, 0.0, 0.0, 0.1);
        let seq1 = sampler.update(p1, 2000);
        assert_eq!(seq1, 1);
        assert_eq!(sampler.active_index(), 1);
        let snap1 = sampler.sample();
        assert_eq!(snap1.pose, p1);
        assert_eq!(snap1.sequence, 1);

        let p2 = PlatformPose::new(1020, 2.0, 2.0, 120.0, 0.0, 0.0, 0.2);
        let seq2 = sampler.update(p2, 3000);
        assert_eq!(seq2, 2);
        assert_eq!(sampler.active_index(), 0);
        let snap2 = sampler.sample();
        assert_eq!(snap2.pose, p2);
    }

    #[test]
    fn test_concurrent_writes_and_reads() {
        use std::sync::Arc;
        use std::thread;
        let p0 = PlatformPose::new(1000, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let sampler = Arc::new(DualBufferSampler::new(p0, 0));
        let sw = Arc::clone(&sampler);
        let w = thread::spawn(move || {
            for i in 1..=200 {
                let p = PlatformPose::new(1000 + i, i as f64, 0.0, 0.0, 0.0, 0.0, 0.0);
                sw.update(p, i * 1000);
            }
        });
        let sr = Arc::clone(&sampler);
        let r = thread::spawn(move || {
            for _ in 0..500 {
                let snap = sr.sample();
                assert!(snap.is_valid);
            }
        });
        w.join().unwrap();
        r.join().unwrap();
        let stats = sampler.stats();
        assert_eq!(stats.writes_total, 200);
        assert!(stats.reads_total >= 500);
    }
}
