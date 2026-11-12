use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

fn filter() -> TrackFilter<ManualClock> {
    let cfg = FilterConfig {
        min_interval_ms: 0,
        min_move_m: 0.0,
        stale_after_ms: 5_000,
    };
    TrackFilter::new(cfg, ManualClock::new(0))
}

#[test]
fn fresh_track_is_retained() {
    let mut f = filter();
    f.accept("t1", 1.0, 1.0);
    f.clock().advance(5_000);
    assert_eq!(f.evict_stale(), 0);
    assert_eq!(f.tracked_count(), 1);
}
