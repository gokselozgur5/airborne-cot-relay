use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn det(u: f64, v: f64, emb: Option<[f32; 8]>) -> TargetDetection {
    let bbox = (u - 10.0, v - 10.0, u + 10.0, v + 10.0);
    let classification = "vehicle".into();
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox,
        classification,
        confidence: 0.9,
        embedding: emb,
    }
}
