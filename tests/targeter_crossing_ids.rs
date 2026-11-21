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

fn ids_per_step(frames: &[Vec<TargetDetection>]) -> Vec<Vec<(u64, f64)>> {
    let mut targeter = Targeter::with_params(&CameraIntrinsics::default(), 0.0, 2, 5, 1.0);
    let mut out = Vec::new();
    for (k, dets) in frames.iter().enumerate() {
        let pose = PlatformPose::new(1000 + k as u64 * 100, 37.0, -122.0, 100.0, 0.0, 0.0, 0.0);
        let ids: Vec<(u64, f64)> = targeter
            .update(&pose, dets)
            .unwrap()
            .iter()
            .filter(|t| t.is_confirmed)
            .map(|t| (t.id, t.velocity_enu[0].signum()))
            .collect();
        out.push(ids);
    }
    out
}

#[test]
fn crossing_targets_keep_two_stable_ids() {
    let frames: Vec<_> = (0..10)
        .map(|k| {
            vec![
                det(400.0 + 30.0 * k as f64, 260.0, None),
                det(880.0 - 30.0 * k as f64, 300.0, None),
            ]
        })
        .collect();
    let steps = ids_per_step(&frames);
    let first_valid = steps.iter().find(|ids| !ids.is_empty()).unwrap();
    assert!(
        steps
            .iter()
            .filter(|ids| !ids.is_empty())
            .all(|ids| ids.len() == 2 && ids == first_valid),
        "ids per step: {steps:?}"
    );
}

#[test]
fn target_moving_30_px_per_frame_stays_associated() {
    let frames: Vec<_> = (0..10)
        .map(|k| vec![det(300.0 + 30.0 * k as f64, 400.0, None)])
        .collect();
    let steps = ids_per_step(&frames);
    let first_valid = steps.iter().find(|ids| !ids.is_empty()).unwrap();
    assert!(
        steps
            .iter()
            .filter(|ids| !ids.is_empty())
            .all(|ids| ids.len() == 1 && ids[0].0 == first_valid[0].0),
        "ids per step: {steps:?}"
    );
}

#[test]
fn appearance_change_keeps_id_with_predicted_box() {
    let (mut a, mut b) = ([0.0f32; 8], [0.0f32; 8]);
    (a[1], b[2]) = (1.0, 1.0);
    let frames: Vec<_> = (0..10)
        .map(|k| {
            let emb = if k < 5 { a } else { b };
            vec![det(300.0 + 30.0 * k as f64, 400.0, Some(emb))]
        })
        .collect();
    let steps = ids_per_step(&frames);
    let first_valid = steps.iter().find(|ids| !ids.is_empty()).unwrap();
    assert!(
        steps
            .iter()
            .filter(|ids| !ids.is_empty())
            .all(|ids| ids.len() == 1 && ids[0].0 == first_valid[0].0),
        "ids per step: {steps:?}"
    );
}
