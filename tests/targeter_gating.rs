use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn intrinsics() -> CameraIntrinsics {
    CameraIntrinsics {
        focal_length_px: 1000.0,
        principal_u: 500.0,
        principal_v: 500.0,
        image_width: 1000,
        image_height: 1000,
    }
}

fn detection(u: f64, v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

#[test]
fn test_temporal_gating_stale_detection_ignored() {
    let mut t = Targeter::new(&intrinsics(), 0.0);
    let mut pose = PlatformPose::new(1000, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0);
    t.update(&pose, &[detection(500.0, 500.0)]).unwrap();
    pose.timestamp_ms = 900;
    let tracks = t.update(&pose, &[detection(500.0, 500.0)]).unwrap();
    assert_eq!(tracks[0].hits, 1);
}
