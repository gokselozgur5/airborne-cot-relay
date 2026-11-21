use airborne_cot_relay::{CotDedupCache, DedupCacheConfig};

#[test]
fn test_duplicate_hits_do_not_grow_expiry_queue() {
    let mut cache = CotDedupCache::new();
    assert!(cache.check_and_insert(b"k1", 1_000));
    assert_eq!(cache.expiry_queue_len(), 1);
    for now in 1_001..=1_100 {
        assert!(!cache.check_and_insert(b"k1", now));
    }
    assert_eq!(cache.expiry_queue_len(), 1);
}

#[test]
fn refreshed_key_is_not_evicted_before_older_unrefreshed_key() {
    let mut cache = CotDedupCache::with_config(DedupCacheConfig {
        sliding_window_ms: 10_000,
        max_capacity: 2,
        bloom_bits: 512,
        bloom_hashes: 3,
    });
    cache.check_and_insert(b"A", 100);
    cache.check_and_insert(b"B", 200);
    cache.check_and_insert(b"A", 500);
    cache.check_and_insert(b"C", 600);
    assert!(
        cache.is_duplicate(b"A", 600),
        "A was refreshed and should not be evicted"
    );
    assert!(
        !cache.is_duplicate(b"B", 600),
        "B had older expiry and should be evicted"
    );
}
