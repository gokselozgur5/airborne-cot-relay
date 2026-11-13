use airborne_cot_relay::ingest::UdpDetectionSource;
use airborne_cot_relay::runtime::source::DetectionSource;
use std::net::UdpSocket;
use std::time::Duration;

#[test]
fn test_udp_detection_source_empty_returns_none() {
    let mut src = UdpDetectionSource::bind("127.0.0.1:0").unwrap();
    assert_eq!(src.poll_batch().unwrap(), None);
    assert_eq!(src.poll().unwrap(), None);
}

#[test]
fn test_parse_detection_line_valid() {
    let line = "100.0, 200.0, 90.0, 190.0, 20.0, 20.0, person, 0.88";
    let d = airborne_cot_relay::ingest::udp_detections::parse_detection_line(line).unwrap();
    assert_eq!(d.classification, "person");
    assert_eq!(d.pixel_u, 100.0);
    assert_eq!(d.bbox, (90.0, 190.0, 20.0, 20.0));
}

#[test]
fn test_udp_detection_source_receives_loopback_batch() {
    let mut src = UdpDetectionSource::bind("127.0.0.1:0").unwrap();
    let addr = src.local_addr().unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let payload =
        b"10.0,20.0,5.0,15.0,10.0,10.0,car,0.9\nbadline\n11.0,21.0,6.0,16.0,10.0,10.0,truck,0.8\n";
    tx.send_to(payload, addr).unwrap();
    for _ in 0..50 {
        if let Some(batch) = src.poll_batch().unwrap() {
            assert_eq!(batch.len(), 2);
            assert_eq!(src.bad_lines(), 1);
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for detection batch");
}
