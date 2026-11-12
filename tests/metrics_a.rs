use airborne_cot_relay::metrics::Metrics;

#[test]
fn counters_saturate_at_max() {
    let m = Metrics::new();
    m.add_packets_in(u64::MAX);
    m.add_packets_in(5);
    m.add_dropped(u64::MAX);
    m.add_dropped(1);
    let s = m.snapshot(0);
    assert_eq!(s.packets_in, u64::MAX);
    assert_eq!(s.dropped, u64::MAX);
}
