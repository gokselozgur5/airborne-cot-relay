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

fn frame(t: &mut Targeter, i: u64, seen: bool) -> Vec<TrackedObject> {
    let dets = if seen {
        vec![det(640.0, 360.0)]
    } else {
        vec![]
    };
    t.update(&pose(1000 + i * 100), &dets).unwrap()
}

#[test]
fn track_is_dropped_after_long_gap() {
    let (mut t, _rx, _s) = rig();
    for i in 0..11 {
        frame(&mut t, i, i < 4);
    }
    assert!(frame(&mut t, 11, false).is_empty());
}

#[test]
fn short_gap_keeps_track_id() {
    let (mut t, _rx, _s) = rig();
    let id = frame(&mut t, 0, true)[0].id;
    frame(&mut t, 1, true);
    frame(&mut t, 2, false);
    frame(&mut t, 3, false);
    let objs = frame(&mut t, 4, true);
    assert_eq!(objs.len(), 1);
    assert_eq!(objs[0].id, id);
    assert!(objs[0].is_confirmed);
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
fn cot_stops_during_gap_and_resumes_on_return() {
    let (mut t, rx, s) = rig();
    let mut sent = 0;
    for i in 0..16u64 {
        let seen = !(4..11).contains(&i);
        for o in frame(&mut t, i, seen).iter().filter(|o| o.is_confirmed) {
            emit(&s, o, T0_MS + (i as i64) * 100);
            sent += 1;
        }
    }
    let mut times = Vec::new();
    for _ in 0..sent {
        times.push(recv(&rx));
    }
    assert!(rx.recv(&mut [0u8; 8]).is_err());
    assert_eq!(sent, 3 + 4);
    assert!(times.iter().all(|x| x.contains("uid=\"TRACK-")));
}

fn attr(xml: &str, name: &str) -> String {
    let key = format!(" {name}=\"");
    let start = xml.find(&key).unwrap() + key.len();
    let len = xml[start..].find('"').unwrap();
    xml[start..start + len].to_string()
}

#[test]
fn event_times_follow_frame_order_across_reacquisition() {
    let (mut t, rx, s) = rig();
    for i in 0..16u64 {
        let seen = !(4..11).contains(&i);
        for o in frame(&mut t, i, seen).iter().filter(|o| o.is_confirmed) {
            emit(&s, o, T0_MS + (i as i64) * 100);
        }
    }
    let mut prev = String::new();
    for _ in 0..7 {
        let time = attr(&recv(&rx), "time");
        assert!(time >= prev);
        prev = time;
    }
}
