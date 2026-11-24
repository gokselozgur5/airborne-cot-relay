//! Telemetry stage: raw bytes to platform poses with packet accounting.

use super::Pipeline;
use crate::clock::Clock;
use crate::PlatformPose;

impl Pipeline {
    /// Feeds raw telemetry bytes through the ingest.
    ///
    /// Each decoded pose adds one to `Metrics::packets_in` and each frame
    /// that fails to decode adds one to `Metrics::dropped`.
    pub fn ingest_telemetry(&mut self, bytes: &[u8]) -> Vec<PlatformPose> {
        let now_ms = crate::clock::SystemClock::new().now_ms();
        self.ingest_telemetry_at(bytes, now_ms)
    }

    /// Feeds raw telemetry bytes through the ingest with an explicit arrival timestamp.
    pub fn ingest_telemetry_at(&mut self, bytes: &[u8], now_ms: u64) -> Vec<PlatformPose> {
        let errors_before = self.ingest.decode_errors();
        let poses = self.ingest.process_bytes_at(bytes, Some(now_ms));
        let failed = self.ingest.decode_errors().saturating_sub(errors_before);
        for pose in &poses {
            self.stale_gate.update_with_arrival(pose, now_ms);
        }
        self.metrics.add_packets_in(poses.len() as u64);
        self.metrics.add_dropped(failed);
        poses
    }

    /// Reference to the underlying telemetry ingest.
    pub fn telemetry_ingest(&self) -> &crate::TelemetryIngest {
        &self.ingest
    }

    /// Mutable reference to the underlying telemetry ingest.
    pub fn telemetry_ingest_mut(&mut self) -> &mut crate::TelemetryIngest {
        &mut self.ingest
    }

    /// Clears unparsed bytes from the telemetry ingest buffer.
    pub fn clear_telemetry_buffer(&mut self) {
        self.ingest.clear_buffer();
    }
}
