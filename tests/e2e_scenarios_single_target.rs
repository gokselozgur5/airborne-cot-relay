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

#[test]
fn target_is_confirmed_on_second_frame() {
    let (mut targeter, _rx, _s) = rig();
    let first = targeter.update(&pose(1000), &[det(640.0, 360.0)]).unwrap();
    assert_eq!(first.len(), 1);
    assert!(!first[0].is_confirmed);
    let second = targeter.update(&pose(1100), &[det(640.0, 360.0)]).unwrap();
    assert!(second[0].is_confirmed);
    assert_eq!(second[0].id, first[0].id);
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
fn emitted_cot_carries_uid_and_type() {
    let (mut targeter, rx, s) = rig();
    for i in 0..2u64 {
        let objs = targeter
            .update(&pose(1000 + i * 100), &[det(640.0, 360.0)])
            .unwrap();
        if objs[0].is_confirmed {
            emit(&s, &objs[0], T0_MS);
        }
    }
    let xml = recv(&rx);
    assert!(xml.contains("uid=\"TRACK-0001\""));
    assert!(xml.contains("type=\"a-f-G-E-V-C\""));
}

fn attr(xml: &str, name: &str) -> String {
    let key = format!(" {name}=\"");
    let start = xml.find(&key).unwrap() + key.len();
    let len = xml[start..].find('"').unwrap();
    xml[start..start + len].to_string()
}

#[test]
fn cot_point_is_under_nadir_platform() {
    let (mut targeter, rx, s) = rig();
    for i in 0..2u64 {
        let objs = targeter
            .update(&pose(1000 + i * 100), &[det(640.0, 360.0)])
            .unwrap();
        if objs[0].is_confirmed {
            emit(&s, &objs[0], T0_MS);
        }
    }
    let xml = recv(&rx);
    let lat: f64 = attr(&xml, "lat").parse().unwrap();
    let lon: f64 = attr(&xml, "lon").parse().unwrap();
    assert!((lat - 37.7749).abs() < 1e-4);
    assert!((lon + 122.4194).abs() < 1e-4);
}

#[test]
fn stale_follows_event_time_and_count_matches_confirmed_frames() {
    let (mut targeter, rx, s) = rig();
    let mut sent = 0;
    for i in 0..5u64 {
        let objs = targeter
            .update(&pose(1000 + i * 100), &[det(640.0, 360.0)])
            .unwrap();
        if objs[0].is_confirmed {
            emit(&s, &objs[0], T0_MS + (i as i64) * 100);
            sent += 1;
        }
    }
    assert_eq!(sent, 4);
    let xml = recv(&rx);
    assert_eq!(attr(&xml, "time"), "2023-11-14T22:13:20.100Z");
    assert_eq!(attr(&xml, "stale"), "2023-11-14T22:13:25.100Z");
    for _ in 1..sent {
        assert!(recv(&rx).contains("TRACK-0001"));
    }
}
