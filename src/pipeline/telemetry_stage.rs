//! Telemetry stage: raw bytes to platform poses with packet accounting.

use super::Pipeline;
use crate::PlatformPose;

impl Pipeline {
    /// Feeds raw telemetry bytes through the ingest.
    ///
    /// Each decoded pose adds one to `Metrics::packets_in`.
    pub fn ingest_telemetry(&mut self, bytes: &[u8]) -> Vec<PlatformPose> {
        let poses = self.ingest.process_bytes(bytes);
        self.metrics.add_packets_in(poses.len() as u64);
        poses
    }
}
