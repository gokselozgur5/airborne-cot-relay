use airborne_cot_relay::ingest::SourceFilter;
use std::net::IpAddr;

#[test]
fn test_source_filter_allowlist_passes_and_disallowed_rejected() {
    let allowed: IpAddr = "192.168.1.10".parse().unwrap();
    let other: IpAddr = "10.0.0.1".parse().unwrap();
    let mut filter = SourceFilter::new(vec![allowed]);

    assert!(filter.check(&allowed));
    assert_eq!(filter.rejected_count(), 0);

    assert!(!filter.check(&other));
    assert_eq!(filter.rejected_count(), 1);

    assert!(!filter.check(&other));
    assert_eq!(filter.rejected_count(), 2);

    assert!(filter.check(&allowed));
    assert_eq!(filter.rejected_count(), 2);
}

#[test]
fn test_source_filter_empty_allowlist_rejects_all() {
    let mut filter = SourceFilter::new(std::iter::empty());
    let addr: IpAddr = "127.0.0.1".parse().unwrap();
    assert!(!filter.check(&addr));
    assert_eq!(filter.rejected_count(), 1);
}
