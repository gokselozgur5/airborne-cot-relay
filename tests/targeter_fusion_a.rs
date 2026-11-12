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

fn detection(u: f64, v: f64, class: &str) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: class.into(),
        confidence: 0.9,
        embedding: None,
    }
}

fn sorted_classes(tracks: &[airborne_cot_relay::TrackedObject]) -> Vec<String> {
    let mut classes: Vec<String> = tracks.iter().map(|t| t.classification.clone()).collect();
    classes.sort_unstable();
    classes
}

#[test]
fn new_tracks_take_their_own_detection_class() {
    let mut targeter = Targeter::new(&intrinsics(), 0.0);
    let pose = PlatformPose::new(1000, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0);
    let dets = [
        detection(300.0, 300.0, "vehicle"),
        detection(700.0, 700.0, "person"),
    ];
    let tracks = targeter.update(&pose, &dets).expect("update failed");
    assert_eq!(sorted_classes(&tracks), ["person", "vehicle"]);
}
