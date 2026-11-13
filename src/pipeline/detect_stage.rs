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
