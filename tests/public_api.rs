//! Smoke tests that exercise every item re-exported from the crate root.

use airborne_cot_relay::{CameraIntrinsics, RelayConfig};

#[test]
fn default_config_validates() {
    let config = RelayConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.target_stale_timeout_ms, 5000);
}

#[test]
fn default_intrinsics_match_documented_values() {
    let intrinsics = CameraIntrinsics::default();
    assert_eq!(intrinsics.focal_length_px, 1200.0);
    assert_eq!(intrinsics.image_width, 1280);
    assert_eq!(intrinsics.image_height, 720);
}

use airborne_cot_relay::{classification_to_cot_type, RelayError};

#[test]
fn classification_maps_to_cot_type() {
    assert_eq!(classification_to_cot_type("Truck"), "a-f-G-E-V-C");
    assert_eq!(classification_to_cot_type("drone"), "a-f-A-M-F");
    assert_eq!(classification_to_cot_type("unknown"), "a-u-G");
}

#[test]
fn relay_error_display_uses_prefix() {
    let err = RelayError::IoError("network down".into());
    assert_eq!(err.to_string(), "IO error: network down");
}

use airborne_cot_relay::{PlatformPose, TelemetryIngest};

#[test]
fn pose_staleness_respects_max_age() {
    let pose = PlatformPose::new(1_000, 37.0, -122.0, 500.0, 0.0, 0.0, 0.0);
    assert!(!pose.is_stale(1_400, 500));
    assert!(pose.is_stale(2_000, 500));
}

#[test]
fn telemetry_ingest_starts_without_pose() {
    let mut ingest = TelemetryIngest::new(37.0, -122.0, 0.0, 1_000);
    assert!(ingest.latest_pose().is_none());
    assert!(ingest.process_bytes(&[]).is_empty());
}
