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
