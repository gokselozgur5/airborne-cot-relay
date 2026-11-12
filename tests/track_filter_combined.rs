use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

#[test]
fn backward_step_after_keepalive_still_accepts_next_event() {
    let cfg = FilterConfig {
        min_interval_ms: 1_000,
        min_move_m: 1000.0,
        stale_after_ms: 60_000,
    };
    let mut f = TrackFilter::new(cfg, ManualClock::new(100_000)).with_keepalive_ms(5_000);
    assert!(f.accept("t1", 10.0, 20.0));
    f.clock().advance(6_000);
    assert!(f.accept("t1", 10.0, 20.0));
    f.clock().set(50_000);
    assert!(f.accept("t1", 10.0, 20.0));
    assert_eq!(f.evict_stale(), 0);
    assert_eq!(f.tracked_count(), 1);
}
