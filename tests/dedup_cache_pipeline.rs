use airborne_cot_relay::{CotDedupCache, DedupCacheConfig};

#[test]
fn test_integration_cot_dedup_cache_pipeline() {
    let mut cache = CotDedupCache::with_config(DedupCacheConfig {
        sliding_window_ms: 2000,
        max_capacity: 100,
        bloom_bits: 4096,
        bloom_hashes: 4,
    });

    for i in 0..50 {
        let uid = format!("track-{}", i);
        assert!(cache.check_and_insert_uid_time(&uid, 100, 1000));
    }
    assert_eq!(cache.len(), 50);

    for i in 0..50 {
        let uid = format!("track-{}", i);
        assert!(!cache.check_and_insert_uid_time(&uid, 100, 1500));
    }
    assert_eq!(cache.stats().duplicates_dropped, 50);

    assert_eq!(cache.prune_expired(4000), 50);
    assert_eq!(cache.len(), 0);
}
