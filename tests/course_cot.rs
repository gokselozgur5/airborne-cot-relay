use airborne_cot_relay::TrackSummary;
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
