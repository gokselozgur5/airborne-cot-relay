use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn targeter() -> Targeter {
    let intrinsics = CameraIntrinsics {
        focal_length_px: 1000.0,
        principal_u: 500.0,
        principal_v: 500.0,
        image_width: 1000,
        image_height: 1000,
    };
    Targeter::with_params(&intrinsics, 0.0, 2, 5, 1.0)
}

fn detection(pixel_v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: 500.0,
        pixel_v,
        bbox: (490.0, 490.0, 510.0, 510.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

#[test]
fn above_horizon_detection_does_not_drop_the_frame() {
    let mut t = targeter();
    let mut pose = PlatformPose::new(1000, 37.7749, -122.4194, 100.0, 0.0, 1.0, 0.0);
    t.update(&pose, &[detection(500.0)])
        .expect("update 1 failed");
    pose.timestamp_ms += 100;
    let tracks = t
        .update(&pose, &[detection(500.0), detection(-2000.0)])
        .expect("valid detection must still be fused");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].hits, 2);
    assert!(tracks[0].is_confirmed);
}

#[test]
fn unusable_pose_still_lets_tracks_coast() {
    let mut t = targeter();
    let mut pose = PlatformPose::new(1000, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0);
    t.update(&pose, &[detection(500.0)])
        .expect("update 1 failed");
    pose.timestamp_ms += 100;
    pose.hae_m = 0.0;
    let tracks = t
        .update(&pose, &[detection(500.0)])
        .expect("altitude zero must not fail the frame");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].hits, 1);
}
