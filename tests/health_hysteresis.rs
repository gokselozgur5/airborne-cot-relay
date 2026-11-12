use airborne_cot_relay::health::Hysteresis;

#[test]
fn commits_after_required_streak() {
    let mut h = Hysteresis::new(0, 3);
    assert_eq!(h.observe(1), 0);
    assert_eq!(h.observe(1), 0);
    assert_eq!(h.observe(1), 1);
    assert_eq!(h.committed(), 1);
}

#[test]
fn interrupted_streak_resets() {
    let mut h = Hysteresis::new(0, 3);
    h.observe(1);
    h.observe(1);
    h.observe(2);
    h.observe(1);
    assert_eq!(h.observe(1), 0);
}

#[test]
fn returning_to_committed_clears_candidate() {
    let mut h = Hysteresis::new(0, 2);
    h.observe(1);
    h.observe(0);
    assert_eq!(h.observe(1), 0);
}
