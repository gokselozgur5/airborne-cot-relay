use airborne_cot_relay::{
    CameraIntrinsics, PlatformPose, TargetDetection, Targeter, TelemetryIngest,
};
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

#[test]
fn right_bank_heading_north_moves_boresight_west() {
    let east = boresight_east_m(0.3, 0.0, 0.0);
    assert!((east + 30.934).abs() < 0.05, "east offset {east}");
}

#[test]
fn heading_and_bank_sweep_finite_ground_intersection() {
    for yaw_deg in [-180, -90, 0, 90, 180] {
        for bank_deg in [-20, -10, 0, 10, 20] {
            let east = boresight_east_m(
                (bank_deg as f32).to_radians(),
                0.0,
                (yaw_deg as f32).to_radians(),
            );
            assert!(east.is_finite(), "east must be finite");
        }
    }
}

#[test]
fn class_map_prunes_stale_track_entries() {
    let cam = CameraIntrinsics::default();
    let mut targeter = Targeter::with_params(&cam, 0.0, 1, 1, 1.0);
    let mut pose = PlatformPose::new(1000, 37.0, -122.0, 100.0, 0.0, 0.0, 0.0);
    let det = TargetDetection {
        pixel_u: cam.principal_u,
        pixel_v: cam.principal_v,
        bbox: (
            cam.principal_u - 10.0,
            cam.principal_v - 10.0,
            cam.principal_u + 10.0,
            cam.principal_v + 10.0,
        ),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    };

    targeter.update(&pose, &[det]).unwrap();
    assert_eq!(targeter.class_map_len(), 1);

    for _ in 0..3 {
        pose.timestamp_ms += 100;
        targeter.update(&pose, &[]).unwrap();
    }
    assert_eq!(targeter.class_map_len(), 0);
}
