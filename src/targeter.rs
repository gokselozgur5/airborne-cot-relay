//! Multi-target fusion and geospatial projection engine.
use crate::config::{CameraIntrinsics, RelayError};
use crate::telemetry::PlatformPose;
use sightline_cot::camera::{
    enu_to_geodetic, estimate_covariance_ce_le, intersect_ground, CameraPose, Intrinsics,
};
use sightline_cot::tracker::{BoundingBox, Detection, MultiTargetTracker, TrackStatus};

/// Raw visual detection with bounding box and classification.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetDetection {
    pub pixel_u: f64,
    pub pixel_v: f64,
    pub bbox: (f64, f64, f64, f64),
    pub classification: String,
    pub confidence: f32,
    pub embedding: Option<[f32; 8]>,
}

/// Projected ground coordinates and covariance bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundIntersection {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub hae_m: f64,
    pub ce90_m: f64,
    pub le90_m: f64,
    pub enu_m: [f64; 3],
}

/// Fused tracked target state ready for CoT relay.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedObject {
    pub id: u64,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub hae_m: f64,
    pub velocity_enu: [f64; 2],
    pub speed_mps: f64,
    pub course_deg: f64,
    pub ce90_m: f64,
    pub classification: String,
    pub hits: usize,
    pub is_confirmed: bool,
}

/// Last measured image-space box centre and per-frame shift of one track.
#[derive(Debug, Clone, Copy, PartialEq)]
struct BoxMotion {
    center: (f64, f64),
    frame: u64,
    shift: (f64, f64),
}

/// Multi-target geospatial projection and tracking engine.
pub struct Targeter {
    camera: Intrinsics,
    ground_elevation_m: f64,
    tracker: MultiTargetTracker,
    last_timestamp_ms: Option<u64>,
    class_map: std::collections::HashMap<u64, String>,
    anchor_deg: Option<(f64, f64)>,
    frame_offset_m: [f64; 2],
    box_motion: std::collections::HashMap<u64, BoxMotion>,
    frame: u64,
}

impl Targeter {
    /// Create a new Targeter instance from intrinsics and ground elevation.
    pub fn new(intrinsics: &CameraIntrinsics, ground_elevation_m: f64) -> Self {
        let cam = Intrinsics::Pinhole {
            fx: intrinsics.focal_length_px,
            fy: intrinsics.focal_length_px,
            cx: intrinsics.principal_u,
            cy: intrinsics.principal_v,
        };
        Self {
            camera: cam,
            ground_elevation_m,
            tracker: MultiTargetTracker::default(),
            last_timestamp_ms: None,
            class_map: std::collections::HashMap::new(),
            anchor_deg: None,
            frame_offset_m: [0.0, 0.0],
            box_motion: std::collections::HashMap::new(),
            frame: 0,
        }
    }
}

impl Targeter {
    /// Create a new Targeter with customized tracker parameters.
    pub fn with_params(
        intrinsics: &CameraIntrinsics,
        ground_elevation_m: f64,
        min_hits: usize,
        max_age: usize,
        q_acc: f64,
    ) -> Self {
        let mut targeter = Self::new(intrinsics, ground_elevation_m);
        targeter.tracker = MultiTargetTracker::new(min_hits, max_age, q_acc);
        targeter
    }
}

impl Targeter {
    fn make_camera_pose(pose: &PlatformPose) -> CameraPose {
        CameraPose {
            roll_rad: pose.roll_rad,
            pitch_rad: pose.pitch_rad,
            yaw_rad: pose.yaw_rad,
            cam_up_m: pose.hae_m,
        }
    }
}

impl Targeter {
    fn project_ray_to_enu(
        &self,
        pixel_u: f64,
        pixel_v: f64,
        cam_pose: &CameraPose,
    ) -> Result<[f64; 3], RelayError> {
        let ray = self
            .camera
            .pixel_to_ray(pixel_u, pixel_v)
            .map_err(|e| RelayError::TargetingError(format!("ray error: {e}")))?;
        intersect_ground(ray, cam_pose, self.ground_elevation_m)
            .map_err(|e| RelayError::TargetingError(format!("ground intersection error: {e}")))
    }
}

impl Targeter {
    fn estimate_cov_ce_le(
        &self,
        cam_pose: &CameraPose,
        pixel_u: f64,
        pixel_v: f64,
    ) -> Result<(f64, f64), RelayError> {
        estimate_covariance_ce_le(
            &self.camera,
            cam_pose,
            pixel_u,
            pixel_v,
            self.ground_elevation_m,
            1.0,
            0.01,
        )
        .map_err(|e| RelayError::TargetingError(format!("covariance error: {e}")))
    }
}

impl Targeter {
    /// Intersect pixel ray with ground elevation and estimate CE90/LE90.
    pub fn project_pixel(
        &self,
        pixel_u: f64,
        pixel_v: f64,
        pose: &PlatformPose,
    ) -> Result<GroundIntersection, RelayError> {
        let cam_pose = Self::make_camera_pose(pose);
        let enu_pt = self.project_ray_to_enu(pixel_u, pixel_v, &cam_pose)?;
        let (ce90, le90) = self.estimate_cov_ce_le(&cam_pose, pixel_u, pixel_v)?;
        let enu_rel = [enu_pt[0], enu_pt[1], enu_pt[2] - pose.hae_m];
        let (lat, lon, hae) = enu_to_geodetic(enu_rel, pose.lat_deg, pose.lon_deg, pose.hae_m);
        Ok(GroundIntersection {
            lat_deg: lat,
            lon_deg: lon,
            hae_m: hae,
            ce90_m: ce90,
            le90_m: le90,
            enu_m: enu_pt,
        })
    }
}

/// Fixed unit embedding for detections without appearance features.
///
/// The tracker always compares embeddings, and a zero vector scores as a
/// non-match, so absent embeddings would only associate on raw box overlap.
/// A shared unit vector makes the appearance check neutral, leaving the
/// kinematic Mahalanobis gate to decide the association.
const NEUTRAL_EMBEDDING: [f32; 8] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

impl Targeter {
    fn build_detection(det: &TargetDetection, gi: &GroundIntersection) -> Detection {
        Detection {
            east_m: gi.enu_m[0],
            north_m: gi.enu_m[1],
            hae_m: gi.hae_m,
            ce90_m: gi.ce90_m,
            le90_m: gi.le90_m,
            bbox: BoundingBox {
                x1: det.bbox.0,
                y1: det.bbox.1,
                x2: det.bbox.2,
                y2: det.bbox.3,
            },
            embedding: det.embedding.unwrap_or(NEUTRAL_EMBEDDING),
        }
    }
}

impl Targeter {
    fn compute_dt_s(&mut self, timestamp_ms: u64) -> f64 {
        match self.last_timestamp_ms {
            None => {
                self.last_timestamp_ms = Some(timestamp_ms);
                0.1
            }
            Some(prev) => {
                self.last_timestamp_ms = Some(timestamp_ms);
                let diff = timestamp_ms.saturating_sub(prev);
                (diff as f64 / 1000.0).clamp(0.001, 5.0)
            }
        }
    }
}

impl Targeter {
    /// Update tracking filter with new visual detections at current pose.
    pub fn update(
        &mut self,
        pose: &PlatformPose,
        detections: &[TargetDetection],
    ) -> Result<Vec<TrackedObject>, RelayError> {
        let dt_s = self.compute_dt_s(pose.timestamp_ms);
        let mut tracker_dets = Vec::with_capacity(detections.len());
        for d in detections {
            if let Ok(gi) = self.project_pixel(d.pixel_u, d.pixel_v, pose) {
                tracker_dets.push(Self::build_detection(d, &gi));
            }
        }
        self.anchor_detections(pose, &mut tracker_dets);
        self.tracker
            .step(dt_s, &tracker_dets)
            .map_err(|e| RelayError::TargetingError(format!("tracker step error: {e}")))?;
        self.extract_tracked_objects(pose, detections)
    }
}

impl Targeter {
    /// Move each track box forward by its last measured per-frame pixel shift.
    fn predict_track_boxes(&mut self) {
        for trk in &mut self.tracker.tracks {
            if let Some(m) = self.box_motion.get(&trk.id) {
                let (du, dv) = m.shift;
                trk.bbox.x1 += du;
                trk.bbox.x2 += du;
                trk.bbox.y1 += dv;
                trk.bbox.y2 += dv;
            }
        }
    }
}

impl Targeter {
    /// Record the image-space motion of tracks updated in the current frame.
    fn record_box_motion(&mut self) {
        self.frame += 1;
        let (frame, tracks) = (self.frame, &self.tracker.tracks);
        self.box_motion
            .retain(|id, _| tracks.iter().any(|t| t.id == *id));
        for trk in tracks.iter().filter(|t| t.time_since_update == 0) {
            let b = &trk.bbox;
            let center = ((b.x1 + b.x2) / 2.0, (b.y1 + b.y2) / 2.0);
            let shift = self.box_motion.get(&trk.id).map_or((0.0, 0.0), |m| {
                let gap = (frame - m.frame) as f64;
                ((center.0 - m.center.0) / gap, (center.1 - m.center.1) / gap)
            });
            let motion = BoxMotion {
                center,
                frame,
                shift,
            };
            self.box_motion.insert(trk.id, motion);
        }
    }
}

impl Targeter {
    /// Shift platform-relative detections into the frame anchored at the first pose.
    fn anchor_detections(&mut self, pose: &PlatformPose, dets: &mut [Detection]) {
        const EARTH_RADIUS_M: f64 = 6378137.0;
        let (lat0, lon0) = *self.anchor_deg.get_or_insert((pose.lat_deg, pose.lon_deg));
        let north = (pose.lat_deg - lat0).to_radians() * EARTH_RADIUS_M;
        let east = (pose.lon_deg - lon0).to_radians() * EARTH_RADIUS_M * lat0.to_radians().cos();
        self.frame_offset_m = [east, north];
        for d in dets.iter_mut() {
            d.east_m += east;
            d.north_m += north;
        }
    }
}

impl Targeter {
    fn compute_kinematics(vx: f64, vy: f64) -> (f64, f64) {
        let speed = (vx * vx + vy * vy).sqrt();
        let course = vy.atan2(vx).to_degrees().rem_euclid(360.0);
        (speed, course)
    }
}

impl Targeter {
    fn compute_track_ce90(p: &[[f64; 4]; 4]) -> f64 {
        let var_xy = (p[0][0] + p[1][1]) * 0.5;
        (2.146 * var_xy.max(0.0).sqrt()).max(1.0)
    }
}

impl Targeter {
    fn resolve_track_geodetic(
        &self,
        trk: &sightline_cot::tracker::Track,
        pose: &PlatformPose,
    ) -> (f64, f64, f64) {
        let [off_e, off_n] = self.frame_offset_m;
        let up = self.ground_elevation_m - pose.hae_m;
        let enu = [trk.x[0] - off_e, trk.x[1] - off_n, up];
        enu_to_geodetic(enu, pose.lat_deg, pose.lon_deg, pose.hae_m)
    }
}

impl Targeter {
    fn resolve_track_class(&self, id: u64) -> String {
        self.class_map
            .get(&id)
            .cloned()
            .unwrap_or_else(|| "unknown".into())
    }
}

impl Targeter {
    fn convert_single_track(
        &self,
        trk: &sightline_cot::tracker::Track,
        pose: &PlatformPose,
    ) -> TrackedObject {
        let (lat, lon, hae) = self.resolve_track_geodetic(trk, pose);
        let (speed, course) = Self::compute_kinematics(trk.x[2], trk.x[3]);
        let ce90 = Self::compute_track_ce90(&trk.p);
        let cls = self.resolve_track_class(trk.id);
        let confirmed = trk.status == TrackStatus::Confirmed;
        TrackedObject {
            id: trk.id,
            lat_deg: lat,
            lon_deg: lon,
            hae_m: hae,
            velocity_enu: [trk.x[2], trk.x[3]],
            speed_mps: speed,
            course_deg: course,
            ce90_m: ce90,
            classification: cls,
            hits: trk.hits,
            is_confirmed: confirmed,
        }
    }
}

impl Targeter {
    fn extract_tracked_objects(
        &mut self,
        pose: &PlatformPose,
        detections: &[TargetDetection],
    ) -> Result<Vec<TrackedObject>, RelayError> {
        self.update_classification_map(detections);
        let mut results = Vec::with_capacity(self.tracker.tracks.len());
        for trk in &self.tracker.tracks {
            results.push(self.convert_single_track(trk, pose));
        }
        Ok(results)
    }
}

impl Targeter {
    fn update_classification_map(&mut self, detections: &[TargetDetection]) {
        for trk in &self.tracker.tracks {
            if self.class_map.contains_key(&trk.id) {
                continue;
            }
            let bbox = (trk.bbox.x1, trk.bbox.y1, trk.bbox.x2, trk.bbox.y2);
            if let Some(det) = detections.iter().find(|d| d.bbox == bbox) {
                self.class_map.insert(trk.id, det.classification.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn sample_intrinsics() -> CameraIntrinsics {
        CameraIntrinsics {
            focal_length_px: 1000.0,
            principal_u: 500.0,
            principal_v: 500.0,
            image_width: 1000,
            image_height: 1000,
        }
    }

    pub fn nadir_pose(hae_m: f64) -> PlatformPose {
        PlatformPose {
            timestamp_ms: 1000,
            lat_deg: 37.7749,
            lon_deg: -122.4194,
            hae_m,
            roll_rad: 0.0,
            pitch_rad: 0.0,
            yaw_rad: 0.0,
        }
    }
}

#[cfg(test)]
mod tests_projection {
    use super::tests::*;
    use super::*;

    #[test]
    fn test_nadir_pixel_projection() {
        let targeter = Targeter::new(&sample_intrinsics(), 0.0);
        let pose = nadir_pose(100.0);
        let pt = targeter
            .project_pixel(500.0, 500.0, &pose)
            .expect("projection failed");
        assert!(pt.enu_m[0].abs() < 1e-4);
        assert!(pt.enu_m[1].abs() < 1e-4);
        assert_eq!(pt.hae_m.round() as i64, 0);
        assert!((pt.lat_deg - pose.lat_deg).abs() < 1e-5);
        assert!((pt.lon_deg - pose.lon_deg).abs() < 1e-5);
    }
}

#[cfg(test)]
mod tests_horizon {
    use super::tests::*;
    use super::*;

    #[test]
    fn test_ray_above_horizon_error() {
        let targeter = Targeter::new(&sample_intrinsics(), 0.0);
        let mut pose = nadir_pose(100.0);
        pose.pitch_rad = std::f64::consts::FRAC_PI_2;
        let res = targeter.project_pixel(500.0, 500.0, &pose);
        assert!(res.is_err());
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn sample_detection() -> TargetDetection {
        TargetDetection {
            pixel_u: 500.0,
            pixel_v: 500.0,
            bbox: (490.0, 490.0, 510.0, 510.0),
            classification: "vehicle".into(),
            confidence: 0.95,
            embedding: None,
        }
    }
}

#[cfg(test)]
mod tests_tracker {
    use super::test_helpers::sample_detection;
    use super::tests::*;
    use super::*;

    #[test]
    fn test_multi_target_tracking_step() {
        let mut targeter = Targeter::with_params(&sample_intrinsics(), 0.0, 2, 5, 1.0);
        let mut pose = nadir_pose(100.0);
        let det = sample_detection();
        let t1 = targeter
            .update(&pose, std::slice::from_ref(&det))
            .expect("update 1 failed");
        assert_eq!(t1.len(), 1);
        assert!(!t1[0].is_confirmed);
        pose.timestamp_ms += 100;
        let t2 = targeter
            .update(&pose, std::slice::from_ref(&det))
            .expect("update 2 failed");
        assert_eq!(t2.len(), 1);
        assert!(t2[0].is_confirmed);
        assert_eq!(t2[0].classification, "vehicle");
    }
}
