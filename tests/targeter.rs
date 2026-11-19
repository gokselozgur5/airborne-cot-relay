use airborne_cot_relay::{CameraIntrinsics, Targeter, TelemetryIngest};
use stm32_navcore::protocol::NavTelemetry;

fn boresight_east_m(roll: f32, pitch: f32, yaw: f32) -> f64 {
    let mut ingest = TelemetryIngest::new(0.0, 0.0, 0.0, 5000);
    let (pos_ned_m, vel_ned_mps, euler_rad) = ([0.0, 0.0, -100.0], [0.0; 3], [roll, pitch, yaw]);
    let telem = NavTelemetry {
        timestamp_ms: 1000,
        pos_ned_m,
        vel_ned_mps,
        euler_rad,
        status: 0,
    };
    let pose = ingest.update_from_nav(&telem);
    let cam = CameraIntrinsics::default();
    let hit = Targeter::new(&cam, 0.0).project_pixel(cam.principal_u, cam.principal_v, &pose);
    hit.unwrap().enu_m[0]
}

#[test]
fn nose_up_heading_east_moves_boresight_east() {
    let east = boresight_east_m(0.0, 10f32.to_radians(), std::f32::consts::FRAC_PI_2);
    assert!((east - 17.633).abs() < 0.05, "east offset {east}");
}
