use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

#[test]
fn target_seen_every_frame_is_not_evicted() {
    let cfg = FilterConfig {
        min_interval_ms: 1_000,
        min_move_m: 1000.0,
        stale_after_ms: 60_000,
    };
    let mut f = TrackFilter::new(cfg, ManualClock::new(0));
    assert!(f.accept("t1", 10.0, 20.0));
    for _ in 0..7 {
        f.clock().advance(10_000);
        assert!(!f.accept("t1", 10.0, 20.0));
    }
    assert_eq!(f.evict_stale(), 0);
    assert_eq!(f.tracked_count(), 1);
}
