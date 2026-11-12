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

#[test]
fn send_age_does_not_underflow() {
    let m = Metrics::new();
    m.record_sent(1_000);
    assert_eq!(m.last_send_age_ms(500), Some(0));
    m.record_sent(u64::MAX);
    assert!(m.last_send_age_ms(u64::MAX).unwrap() <= 1);
}
