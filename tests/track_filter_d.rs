use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

fn filter(interval: u64) -> TrackFilter<ManualClock> {
    let cfg = FilterConfig {
        min_interval_ms: interval,
        min_move_m: 0.0,
        stale_after_ms: 60_000,
    };
    TrackFilter::new(cfg, ManualClock::new(100))
}

#[test]
fn uids_are_independent() {
    let mut f = filter(1_000);
    assert!(f.accept("a", 1.0, 1.0));
    assert!(f.accept("b", 1.0, 1.0));
    assert!(!f.accept("a", 2.0, 2.0));
    assert_eq!(f.tracked_count(), 2);
}

#[test]
fn frozen_clock_drops_repeats() {
    let mut f = filter(1);
    assert!(f.accept("a", 1.0, 1.0));
    assert!(!f.accept("a", 2.0, 2.0));
}

#[test]
fn zero_interval_accepts_every_event() {
    let mut f = filter(0);
    assert!(f.accept("a", 1.0, 1.0));
    assert!(f.accept("a", 1.0, 1.0));
}
