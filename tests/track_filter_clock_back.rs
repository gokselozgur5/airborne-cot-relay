use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};

#[test]
fn backward_clock_step_does_not_silence_uid() {
    let cfg = FilterConfig {
        min_interval_ms: 1_000,
        min_move_m: 0.0,
        stale_after_ms: 60_000,
    };
    let mut f = TrackFilter::new(cfg, ManualClock::new(100_000));
    assert!(f.accept("t1", 10.0, 20.0));
    f.clock().set(2_000);
    assert!(f.accept("t1", 10.1, 20.1));
}
