use airborne_cot_relay::bloom_dedup::*;

#[test]
fn test_bloom_filter_false_positive_bound() {
    let mut filter = BloomFilter::new(4096, 4);
    for i in 0..100 {
        let key = format!("entity-{}", i);
        filter.insert(key.as_bytes());
    }

    for i in 0..100 {
        let key = format!("entity-{}", i);
        assert!(filter.contains(key.as_bytes()));
    }
}

#[test]
fn test_cot_deduplicator_packet_burst() {
    let mut dedup = CotBloomDeduplicator::new(2048, 4);

    for _ in 0..5 {
        let is_new = dedup.check_and_record_uid_time("RECON-99", "2026-11-16T12:00:00Z");
        assert_eq!(is_new, dedup.total_checked() == 1);
    }

    assert_eq!(dedup.total_checked(), 5);
    assert_eq!(dedup.unique_passed(), 1);
    assert_eq!(dedup.duplicates_dropped(), 4);
}
