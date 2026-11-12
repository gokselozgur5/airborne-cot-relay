use airborne_cot_relay::{CameraIntrinsics, PlatformPose, Targeter};

fn targeter() -> Targeter {
    let intrinsics = CameraIntrinsics {
        focal_length_px: 1000.0,
        principal_u: 500.0,
        principal_v: 500.0,
        image_width: 1000,
        image_height: 1000,
    };
    Targeter::new(&intrinsics, 0.0)
}

fn pitched_pose(pitch_rad: f64) -> PlatformPose {
    PlatformPose::new(1000, 37.7749, -122.4194, 100.0, 0.0, pitch_rad, 0.0)
}

#[test]
fn near_horizon_projection_does_not_claim_tight_ce90() {
    let pose = pitched_pose(std::f64::consts::FRAC_PI_2 - 5e-5);
    if let Ok(gi) = targeter().project_pixel(500.0, 500.0, &pose) {
        let range_m = gi.enu_m[1];
        assert!(gi.ce90_m > 1000.0, "ce90 {} m at {range_m} m", gi.ce90_m);
    }
}
