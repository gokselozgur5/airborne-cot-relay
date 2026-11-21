use airborne_cot_relay::ingest::UdpDetectionSource;
use airborne_cot_relay::runtime::source::DetectionSource;
use std::net::UdpSocket;
use std::time::Duration;

#[test]
fn test_udp_detections_non_utf8_keeps_valid_lines() {
    let mut src = UdpDetectionSource::bind("127.0.0.1:0").unwrap();
    let addr = src.local_addr().unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let payload =
        b"10.0,20.0,5.0,15.0,10.0,10.0,car,0.9\n\xff\n11.0,21.0,6.0,16.0,10.0,10.0,truck,0.8\n";
    tx.send_to(payload, addr).unwrap();
    for _ in 0..50 {
        if let Some(batch) = src.poll_batch().unwrap() {
            assert_eq!(batch.len(), 2);
            assert_eq!(batch[0].classification, "car");
            assert_eq!(batch[1].classification, "truck");
            assert_eq!(src.bad_lines(), 1);
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for detection batch");
}
