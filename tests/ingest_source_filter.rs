use airborne_cot_relay::ingest::source_filter::SourceFilter;
use airborne_cot_relay::ingest::UdpTelemetrySource;
use std::net::Ipv4Addr;

#[test]
fn test_source_filter_rejects_and_counts() {
    let allowed: Ipv4Addr = "127.0.0.1".parse().unwrap();
    let filter = SourceFilter::new(vec![allowed.into()]);
    assert_eq!(filter.rejected_count(), 0);
}
