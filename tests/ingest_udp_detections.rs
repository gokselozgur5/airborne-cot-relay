use airborne_cot_relay::ingest::UdpDetectionSource;
use airborne_cot_relay::runtime::source::DetectionSource;

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
