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

use airborne_cot_relay::{
    CotStreamer, GroundIntersection, TargetDetection, Targeter, TrackSummary, TrackedObject,
};

#[test]
fn targeter_accepts_empty_frame() {
    let mut targeter = Targeter::new(&CameraIntrinsics::default(), 0.0);
    let pose = PlatformPose::new(1_000, 37.0, -122.0, 500.0, 0.0, 0.0, 0.0);
    let tracks = targeter
        .update(&pose, &[])
        .expect("empty frame must not fail");
    assert!(tracks.is_empty());
}

#[test]
fn cot_streamer_builds_event_with_track_uid() {
    let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:6969", 5_000).expect("bind loopback");
    assert_eq!(streamer.target_addr().port(), 6969);
    let track = TrackSummary {
        id: 7,
        classification: "vehicle".into(),
        lat_deg: 37.0,
        lon_deg: -122.0,
        hae_m: 0.0,
        ce90_m: 5.0,
        le90_m: 10.0,
        course_deg: 90.0,
        speed_mps: 12.0,
    };
    let xml = streamer.build_cot_xml(&track, 0);
    assert!(xml.contains(r#"uid="TRACK-0007""#));
    assert!(xml.contains(r#"type="a-f-G-E-V-C""#));
}
