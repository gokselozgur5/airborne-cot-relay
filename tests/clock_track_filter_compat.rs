use airborne_cot_relay::clock::{Clock, ManualClock};
use airborne_cot_relay::track_filter::{self, FilterConfig, TrackFilter};
use std::sync::Arc;

fn config() -> FilterConfig {
    FilterConfig {
        min_interval_ms: 1_000,
        min_move_m: 0.0,
        stale_after_ms: 60_000,
    }
}

#[test]
fn filter_accepts_unified_manual_clock() {
    let mut filter = TrackFilter::new(config(), ManualClock::new(0));
    assert!(filter.accept("t1", 10.0, 20.0));
    filter.clock().advance_ms(999);
    assert!(!filter.accept("t1", 10.1, 20.1));
    filter.clock().advance_ms(1);
    assert!(filter.accept("t1", 10.1, 20.1));
}

#[test]
fn filter_accepts_shared_and_borrowed_clocks() {
    let shared = Arc::new(ManualClock::new(0));
    let mut by_arc = TrackFilter::new(config(), Arc::clone(&shared));
    let mut by_ref = TrackFilter::new(config(), &*shared);
    assert!(by_arc.accept("t1", 10.0, 20.0));
    assert!(by_ref.accept("t1", 10.0, 20.0));
    shared.advance_ms(1_000);
    assert!(by_arc.accept("t1", 10.1, 20.1));
    assert!(by_ref.accept("t1", 10.1, 20.1));
}
