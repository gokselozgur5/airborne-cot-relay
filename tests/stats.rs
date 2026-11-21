use airborne_cot_relay::pipeline::stats::PipelineStatsTracker;
use std::time::{Duration, Instant};

#[test]
fn test_throughput_pps_windowed_repro() {
    let mut tracker = PipelineStatsTracker::new(1.0);
    let t0 = Instant::now();
    tracker.record_received_at(100, t0);
    let t1 = t0 + Duration::from_secs(60);
    tracker.record_received_at(100, t1);
    let pps = tracker.throughput_pps_at(t1);
    assert!(
        (pps - 100.0).abs() < 1e-6,
        "expected 100.0 pps for 1.0s window, got {pps}"
    );
}
