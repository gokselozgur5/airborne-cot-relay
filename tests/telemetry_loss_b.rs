use airborne_cot_relay::telemetry::interpolate_pose;
use airborne_cot_relay::PlatformPose;

fn pose(ts: u64, lat: f64, yaw: f64) -> PlatformPose {
    PlatformPose::new(ts, lat, 10.0, 100.0, 0.0, 0.0, yaw)
}

#[test]
fn midpoint_is_linear() {
    let r = interpolate_pose(&pose(1000, 10.0, 0.0), &pose(2000, 12.0, 1.0), 1500);
    assert_eq!(r.timestamp_ms, 1500);
    assert!((r.lat_deg - 11.0).abs() < 1e-9);
    assert!((r.yaw_rad - 0.5).abs() < 1e-9);
}
