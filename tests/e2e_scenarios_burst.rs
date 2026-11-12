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
fn datagram_count_matches_confirmed_tracks() {
    let (mut t, rx, s) = rig();
    let mut sent = 0;
    for i in 0..60u64 {
        let objs = t.update(&pose(1000 + i * 100), &burst_dets()).unwrap();
        for o in objs.iter().filter(|o| o.is_confirmed) {
            emit(&s, o, T0_MS + (i as i64) * 100);
            sent += 1;
        }
    }
    assert_eq!(sent, 59 * 4);
    for _ in 0..sent {
        recv(&rx);
    }
    assert!(rx.recv(&mut [0u8; 8]).is_err());
}

fn attr(xml: &str, name: &str) -> String {
    let key = format!(" {name}=\"");
    let start = xml.find(&key).unwrap() + key.len();
    let len = xml[start..].find('"').unwrap();
    xml[start..start + len].to_string()
}

#[test]
fn burst_event_times_never_decrease() {
    let (mut t, rx, s) = rig();
    for i in 0..60u64 {
        let objs = t.update(&pose(1000 + i * 100), &burst_dets()).unwrap();
        for o in objs.iter().filter(|o| o.is_confirmed) {
            emit(&s, o, T0_MS + (i as i64) * 100);
        }
    }
    let mut prev = String::new();
    for _ in 0..59 * 4 {
        let time = attr(&recv(&rx), "time");
        assert!(time >= prev);
        prev = time;
    }
}

#[test]
fn each_frame_emits_four_distinct_uids() {
    let (mut t, rx, s) = rig();
    for i in 0..10u64 {
        let objs = t.update(&pose(1000 + i * 100), &burst_dets()).unwrap();
        let confirmed: Vec<_> = objs.iter().filter(|o| o.is_confirmed).collect();
        confirmed.iter().for_each(|o| emit(&s, o, T0_MS));
        let uids: std::collections::BTreeSet<_> =
            confirmed.iter().map(|_| attr(&recv(&rx), "uid")).collect();
        assert_eq!(uids.len(), confirmed.len());
    }
}
