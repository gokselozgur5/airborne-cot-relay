use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

fn filter() -> TrackFilter<ManualClock> {
    let cfg = FilterConfig {
        min_interval_ms: 1_000,
        min_move_m: 0.0,
        stale_after_ms: 60_000,
    };
    TrackFilter::new(cfg, ManualClock::new(0))
}

#[test]
fn first_event_is_accepted() {
    assert!(filter().accept("t1", 10.0, 20.0));
}

