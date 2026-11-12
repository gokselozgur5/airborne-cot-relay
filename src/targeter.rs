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

/// Multi-target geospatial projection and tracking engine.
pub struct Targeter {
    camera: Intrinsics,
    ground_elevation_m: f64,
    tracker: MultiTargetTracker,
    last_timestamp_ms: Option<u64>,
    class_map: std::collections::HashMap<u64, String>,
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
    fn estimate_cov_ce_le(&self, cam_pose: &CameraPose, pixel_u: f64, pixel_v: f64) -> (f64, f64) {
        estimate_covariance_ce_le(
            &self.camera,
            cam_pose,
            pixel_u,
            pixel_v,
            self.ground_elevation_m,
            1.0,
            0.01,
        )
        .unwrap_or((5.0, 5.0))
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
        let (ce90, le90) = self.estimate_cov_ce_le(&cam_pose, pixel_u, pixel_v);
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
            embedding: det.embedding.unwrap_or([0.0; 8]),
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
            let gi = self.project_pixel(d.pixel_u, d.pixel_v, pose)?;
            tracker_dets.push(Self::build_detection(d, &gi));
        }
        self.tracker
            .step(dt_s, &tracker_dets)
            .map_err(|e| RelayError::TargetingError(format!("tracker step error: {e}")))?;
        self.extract_tracked_objects(pose, detections)
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
        let enu = [trk.x[0], trk.x[1], self.ground_elevation_m - pose.hae_m];
        enu_to_geodetic(enu, pose.lat_deg, pose.lon_deg, pose.hae_m)
    }
}
