use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

#[test]
fn unchanged_target_is_resent_after_keepalive() {
    let cfg = FilterConfig {
        min_interval_ms: 1_000,
        min_move_m: 1000.0,
        stale_after_ms: 60_000,
    };
    let mut f = TrackFilter::new(cfg, ManualClock::new(0)).with_keepalive_ms(5_000);
    assert!(f.accept("t1", 10.0, 20.0));
    f.clock().advance(2_000);
    assert!(!f.accept("t1", 10.0, 20.0));
    f.clock().advance(4_000);
    assert!(f.accept("t1", 10.0, 20.0));
}
