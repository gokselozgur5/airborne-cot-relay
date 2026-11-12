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
