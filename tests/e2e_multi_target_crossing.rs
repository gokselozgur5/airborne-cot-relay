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

fn crossing(i: u64) -> Vec<TargetDetection> {
    let k = i as f64 * 4.0;
    vec![det(400.0 + k, 340.0), det(440.0 - k, 380.0)]
}

#[test]
fn two_targets_get_distinct_confirmed_ids() {
    let (mut targeter, _rx, _s) = rig();
    targeter.update(&pose(1000), &crossing(0)).unwrap();
    let objs = targeter.update(&pose(1100), &crossing(1)).unwrap();
    assert_eq!(objs.len(), 2);
    assert!(objs.iter().all(|o| o.is_confirmed));
    assert_ne!(objs[0].id, objs[1].id);
}

fn emit(s: &CotStreamer, o: &TrackedObject, now_ms: i64) {
    let t = TrackSummary {
        id: o.id,
        classification: o.classification.clone(),
        lat_deg: o.lat_deg,
        lon_deg: o.lon_deg,
        hae_m: o.hae_m,
        ce90_m: o.ce90_m,
        le90_m: 10.0,
        course_deg: o.course_deg,
        speed_mps: o.speed_mps,
    };
    s.send_track(&t, now_ms).unwrap();
}

fn recv(rx: &UdpSocket) -> String {
    let mut buf = [0u8; 2048];
    let n = rx.recv(&mut buf).unwrap();
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

#[test]
fn crossing_emits_two_datagrams_per_frame_with_two_uids() {
    let (mut targeter, rx, s) = rig();
    let mut uids = std::collections::BTreeSet::new();
    for i in 0..21u64 {
        let objs = targeter
            .update(&pose(1000 + i * 100), &crossing(i))
            .unwrap();
        assert_eq!(objs.len(), 2);
        for o in objs.iter().filter(|o| o.is_confirmed) {
            emit(&s, o, T0_MS + (i as i64) * 100);
            let xml = recv(&rx);
            assert!(xml.contains(&format!("TRACK-{:04}", o.id)));
            uids.insert(o.id);
        }
    }
    assert_eq!(uids.len(), 2);
}
