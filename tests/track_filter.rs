use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::RelayConfig;

#[test]
fn stationary_track_is_resent_before_its_cot_event_goes_stale() {
    let stale_ms = RelayConfig::default().target_stale_timeout_ms;
    let mut filter = TrackFilter::new(FilterConfig::default(), ManualClock::new(0));
    let mut last_sent_ms = 0u64;
    let mut longest_gap_ms = 0u64;
    for now_ms in (0..=120_000u64).step_by(100) {
        filter.clock().set(now_ms);
        if filter.accept("TRACK-0001", 37.0, -122.0) {
            longest_gap_ms = longest_gap_ms.max(now_ms - last_sent_ms);
            last_sent_ms = now_ms;
        }
    }
    longest_gap_ms = longest_gap_ms.max(120_000 - last_sent_ms);
    assert!(
        longest_gap_ms < stale_ms,
        "longest gap between sends {longest_gap_ms} ms, CoT stale after {stale_ms} ms"
    );
}
