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

fn ids_per_step(frames: &[Vec<TargetDetection>]) -> Vec<Vec<u64>> {
    let mut targeter = Targeter::with_params(&CameraIntrinsics::default(), 0.0, 2, 5, 1.0);
    let mut out = Vec::new();
    for (k, dets) in frames.iter().enumerate() {
        let pose = PlatformPose::new(1000 + k as u64 * 100, 37.0, -122.0, 100.0, 0.0, 0.0, 0.0);
        let mut ids: Vec<u64> = targeter
            .update(&pose, dets)
            .unwrap()
            .iter()
            .map(|t| t.id)
            .collect();
        ids.sort_unstable();
        out.push(ids);
    }
    out
}
