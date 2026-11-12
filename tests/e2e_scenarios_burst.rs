use airborne_cot_relay::{
    CameraIntrinsics, CotStreamer, PlatformPose, TargetDetection, Targeter, TrackSummary,
    TrackedObject,
};
use std::net::UdpSocket;
use std::time::Duration;

const T0_MS: i64 = 1_700_000_000_000;

fn pose(ts_ms: u64) -> PlatformPose {
    PlatformPose::new(ts_ms, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0)
}

fn det(u: f64, v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}
fn rig() -> (Targeter, UdpSocket, CotStreamer) {
    let cam = CameraIntrinsics::default();
    let targeter = Targeter::with_params(&cam, 0.0, 2, 5, 1.0);
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    rx.set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let addr = rx.local_addr().unwrap().to_string();
    let streamer = CotStreamer::new("127.0.0.1:0", &addr, 5000).unwrap();
    (targeter, rx, streamer)
}

fn burst_dets() -> Vec<TargetDetection> {
    [300.0, 500.0, 700.0, 900.0]
        .iter()
        .map(|u| det(*u, 360.0))
        .collect()
}

#[test]
fn sixty_frame_burst_keeps_four_confirmed_tracks() {
    let (mut t, _rx, _s) = rig();
    let mut last = Vec::new();
    for i in 0..60u64 {
        last = t.update(&pose(1000 + i * 100), &burst_dets()).unwrap();
    }
    assert_eq!(last.len(), 4);
    assert!(last.iter().all(|o| o.is_confirmed));
}
