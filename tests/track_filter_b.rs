use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

fn filter() -> TrackFilter<ManualClock> {
    let cfg = FilterConfig {
        min_interval_ms: 0,
        min_move_m: 10.0,
        stale_after_ms: 60_000,
    };
    TrackFilter::new(cfg, ManualClock::new(0))
}

#[test]
fn small_move_is_dropped() {
    let mut f = filter();
    assert!(f.accept("t1", 10.0, 20.0));
    assert!(!f.accept("t1", 10.00001, 20.0));
}
