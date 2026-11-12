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

#[test]
fn clamps_before_first_and_after_last_sample() {
    let (a, b) = (pose(1000, 10.0, 0.0), pose(2000, 12.0, 1.0));
    assert_eq!(interpolate_pose(&a, &b, 500), a);
    assert_eq!(interpolate_pose(&a, &b, 1000), a);
    assert_eq!(interpolate_pose(&a, &b, 2000), b);
    assert_eq!(interpolate_pose(&a, &b, 9000), b);
}

#[test]
fn degenerate_equal_timestamps_return_first() {
    let (a, b) = (pose(1000, 10.0, 0.0), pose(1000, 12.0, 1.0));
    assert_eq!(interpolate_pose(&a, &b, 1000), a);
}

#[test]
fn yaw_takes_shortest_arc_across_wrap() {
    let a = pose(0, 10.0, 3.0);
    let b = pose(1000, 10.0, -3.0);
    let r = interpolate_pose(&a, &b, 500);
    let wrapped = r.yaw_rad.rem_euclid(std::f64::consts::TAU);
    assert!((wrapped - std::f64::consts::PI).abs() < 0.2);
}
