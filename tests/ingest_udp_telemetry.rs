use airborne_cot_relay::ingest::UdpTelemetrySource;
use airborne_cot_relay::runtime::source::TelemetrySource;
use std::net::UdpSocket;
use std::time::Duration;

#[test]
fn test_udp_telemetry_source_empty_socket_returns_none() {
    let mut src = UdpTelemetrySource::bind("127.0.0.1:0").unwrap();
    assert_eq!(src.poll_chunk().unwrap(), None);
    assert_eq!(src.poll().unwrap(), None);
}

#[test]
fn test_udp_telemetry_source_receives_loopback_datagram() {
    let mut src = UdpTelemetrySource::bind("127.0.0.1:0").unwrap();
    let addr = src.local_addr().unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
    tx.send_to(b"telemetry-data", addr).unwrap();
    for _ in 0..50 {
        if let Some(chunk) = src.poll_chunk().unwrap() {
            assert_eq!(chunk, b"telemetry-data");
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for datagram");
}
