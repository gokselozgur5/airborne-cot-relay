use airborne_cot_relay::CotDedupCache;

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
