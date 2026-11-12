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

#[test]
fn stale_track_is_evicted() {
    let mut f = filter();
    f.accept("t1", 1.0, 1.0);
    f.clock().advance(5_001);
    assert_eq!(f.evict_stale(), 1);
    assert_eq!(f.tracked_count(), 0);
}

#[test]
fn evicted_uid_is_accepted_again() {
    let mut f = filter();
    assert!(f.accept("t1", 1.0, 1.0));
    f.clock().advance(6_000);
    f.evict_stale();
    assert!(f.accept("t1", 1.0, 1.0));
}
