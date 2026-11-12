//! Multi-target fusion and geospatial projection engine.
use crate::config::{CameraIntrinsics, RelayError};
use crate::telemetry::PlatformPose;
use sightline_cot::camera::{
    enu_to_geodetic, estimate_covariance_ce_le, intersect_ground, CameraPose, Intrinsics,
};
use sightline_cot::tracker::{BoundingBox, Detection, MultiTargetTracker, TrackStatus};
