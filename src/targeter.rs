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
