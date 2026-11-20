use airborne_cot_relay::ingest::{SourceFilter, UdpTelemetrySource};
use airborne_cot_relay::runtime::source::TelemetrySource;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::time::Duration;

#[test]
fn test_udp_telemetry_source_filters_sender() {
    let filter = SourceFilter::new([IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))]);
    let mut src = UdpTelemetrySource::bind("127.0.0.1:0")
        .unwrap()
        .with_source_filter(filter);
    let addr = src.local_addr().unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
    tx.send_to(b"telemetry-payload", addr).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(src.poll_chunk().unwrap(), None);
    assert_eq!(src.rejected_count(), 1);
}
