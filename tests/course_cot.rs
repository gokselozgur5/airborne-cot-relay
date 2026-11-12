use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter, TrackSummary};
use sightline_cot::tracker::{BoundingBox, Detection, Track};

#[rustfmt::skip]
fn course(ve: f64, vn: f64) -> f64 {
    let bbox = BoundingBox { x1: 0.0, y1: 0.0, x2: 1.0, y2: 1.0 };
    let det = Detection { east_m: 100.0, north_m: 0.0, hae_m: 0.0, ce90_m: 5.0, le90_m: 5.0, bbox, embedding: [0.0; 8] };
    let mut track = Track::new(1, &det);
    track.x[2] = ve;
    track.x[3] = vn;
    TrackSummary::from_track(&track, "vehicle", 37.0, -122.0, 0.0).course_deg
}

#[test]
fn summary_course_follows_velocity_north() {
    assert!((course(0.0, 10.0) - 0.0).abs() < 1e-6);
}

#[test]
fn summary_course_follows_velocity_east() {
    assert!((course(10.0, 0.0) - 90.0).abs() < 1e-6);
}

#[test]
#[rustfmt::skip]
fn targeter_course_is_compass_bearing_north() {
    let k = CameraIntrinsics { focal_length_px: 1000.0, principal_u: 500.0, principal_v: 500.0, image_width: 1000, image_height: 1000 };
    let mut targeter = Targeter::new(&k, 0.0);
    let mut last = Vec::new();
    for i in 0..10u32 {
        let pose = PlatformPose::new(1000 + u64::from(i) * 100, 37.0, -122.0, 100.0, 0.0, 0.0, 0.0);
        let v = 500.0 - 10.0 * f64::from(i);
        let bbox = (490.0, v - 10.0, 510.0, v + 10.0);
        let det = TargetDetection { pixel_u: 500.0, pixel_v: v, bbox, classification: "vehicle".into(), confidence: 0.9, embedding: Some([1.0; 8]) };
        last = targeter.update(&pose, &[det]).expect("update failed");
    }
    let c = last[0].course_deg;
    assert!(c < 10.0 || c > 350.0, "course was {c}");
}
