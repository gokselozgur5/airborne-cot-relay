//! Conversion from fused targeter tracks to relay track summaries.

use crate::{GroundIntersection, TrackSummary, TrackedObject};

/// Builds the relay summary for `track`.
///
/// Position, speed, course, CE90 and class are copied from the track.
/// LE90 comes from `projection` because tracks carry no vertical error.
pub fn track_summary(track: &TrackedObject, projection: &GroundIntersection) -> TrackSummary {
    TrackSummary {
        id: track.id,
        classification: track.classification.clone(),
        lat_deg: track.lat_deg,
        lon_deg: track.lon_deg,
        hae_m: track.hae_m,
        ce90_m: track.ce90_m,
        le90_m: projection.le90_m,
        course_deg: track.course_deg,
        speed_mps: track.speed_mps,
    }
}

impl super::Pipeline {
    /// Summarizes `tracks` with LE90 from the boresight projection at `pose`.
    pub fn summarize_tracks(
        &self,
        tracks: &[TrackedObject],
        pose: &crate::PlatformPose,
    ) -> Result<Vec<TrackSummary>, crate::RelayError> {
        let cam = &self.config.camera;
        let boresight = self
            .targeter
            .project_pixel(cam.principal_u, cam.principal_v, pose)?;
        let summarize = |t| track_summary(t, &boresight);
        Ok(tracks.iter().map(summarize).collect())
    }
}
