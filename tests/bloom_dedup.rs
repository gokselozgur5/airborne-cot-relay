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

#[test]
fn test_cot_deduplicator_reset_window() {
    let mut dedup = CotBloomDeduplicator::new(1024, 3);
    assert!(dedup.check_and_record_uid_time("DRONE-1", "10:00:00"));
    assert!(!dedup.check_and_record_uid_time("DRONE-1", "10:00:00"));

    dedup.reset_window();
    assert!(dedup.check_and_record_uid_time("DRONE-1", "10:00:00"));
}

#[test]
fn never_seen_events_still_pass_after_a_long_run() {
    let mut dedup = CotBloomDeduplicator::new(65_536, 4);
    for i in 0..100_000u32 {
        dedup.check_and_record_uid_time(&format!("UAV-{}", i % 50), &format!("t{i}"));
    }
    let passed = (0..1000)
        .filter(|i| dedup.check_and_record_uid_time(&format!("NEW-{i}"), "2027-01-01T00:00:00Z"))
        .count();
    assert!(passed >= 900, "only {passed} of 1000 never-seen events passed");
}

