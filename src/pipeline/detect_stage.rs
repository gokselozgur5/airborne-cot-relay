//! Detection stage: camera detections fused with a fresh platform pose.

use super::Pipeline;
use crate::{PlatformPose, RelayError, TargetDetection, TrackedObject};

impl Pipeline {
    /// Fuses `detections` with `pose` when it is fresh at `now_ms`.
    ///
    /// With no pose, or a pose older than `target_stale_timeout_ms`, no
    /// tracks are returned and each detection adds one to `Metrics::dropped`.
    pub fn fuse_detections(
        &mut self,
        pose: Option<&PlatformPose>,
        detections: &[TargetDetection],
        now_ms: u64,
    ) -> Result<Vec<TrackedObject>, RelayError> {
        let max_age_ms = self.config.target_stale_timeout_ms;
        match pose.filter(|p| !p.is_stale(now_ms, max_age_ms)) {
            Some(p) => self.targeter.update(p, detections),
            None => {
                self.metrics.add_dropped(detections.len() as u64);
                Ok(Vec::new())
            }
        }
    }
}

impl Pipeline {
    /// Fuses `detections` with the latest pose decoded by the telemetry stage.
    pub fn process_detections(
        &mut self,
        detections: &[TargetDetection],
        now_ms: u64,
    ) -> Result<Vec<TrackedObject>, RelayError> {
        let pose = self.ingest.latest_pose();
        self.fuse_detections(pose.as_ref(), detections, now_ms)
    }

    /// Fuses `detections` with a pose interpolated between `a` and `b` at `now_ms`.
    pub fn fuse_interpolated(
        &mut self,
        a: &PlatformPose,
        b: &PlatformPose,
        detections: &[TargetDetection],
        now_ms: u64,
    ) -> Result<Vec<TrackedObject>, RelayError> {
        let pose = crate::telemetry::interpolate_pose(a, b, now_ms);
        self.fuse_detections(Some(&pose), detections, now_ms)
    }

    /// Returns `true` when the pipeline holds a pose fresher than `target_stale_timeout_ms`.
    pub fn has_fresh_pose(&self, now_ms: u64) -> bool {
        let max_age_ms = self.config.target_stale_timeout_ms;
        self.ingest
            .latest_pose()
            .map_or(false, |p| !p.is_stale(now_ms, max_age_ms))
    }
}
