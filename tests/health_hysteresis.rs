use airborne_cot_relay::health::Hysteresis;

#[test]
fn commits_after_required_streak() {
    let mut h = Hysteresis::new(0, 3);
    assert_eq!(h.observe(1), 0);
    assert_eq!(h.observe(1), 0);
    assert_eq!(h.observe(1), 1);
    assert_eq!(h.committed(), 1);
}
