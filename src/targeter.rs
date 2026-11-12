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
