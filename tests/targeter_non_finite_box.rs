use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn det(u: f64, v: f64) -> TargetDetection {
    let bbox = (u - 10.0, v - 10.0, u + 10.0, v + 10.0);
    let classification = "vehicle".into();
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox,
        classification,
        confidence: 0.9,
        embedding: None,
    }
}

#[test]
fn one_detection_with_a_non_finite_box_does_not_fail_the_frame() {
    let mut targeter = Targeter::new(&CameraIntrinsics::default(), 0.0);
    let pose = PlatformPose::new(1000, 37.0, -122.0, 100.0, 0.0, 0.0, 0.0);
    let mut bad = det(700.0, 400.0);
    bad.bbox.2 = f64::NAN;
    let tracks = targeter.update(&pose, &[det(400.0, 300.0), bad]);
    assert_eq!(tracks.map(|t| t.len()), Ok(1));
}
