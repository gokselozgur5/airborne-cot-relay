use airborne_cot_relay::pipeline::stats::PipelineStatsTracker;

#[test]
fn test_pipeline_stats_tracker_throughput_and_drop_rate() {
    let mut tracker = PipelineStatsTracker::new(1.0);
    tracker.record_received(100);
    tracker.record_dropped(10);
    assert_eq!(tracker.total_received(), 100);
    assert_eq!(tracker.total_dropped(), 10);
    let drop_rate = tracker.drop_rate();
    assert!((drop_rate - 0.10).abs() < 1e-6);
}

#[test]
fn test_pipeline_stats_tracker_throughput_and_window() {
    let mut tracker = PipelineStatsTracker::new(2.0);
    tracker.record_received(50);
    assert!((tracker.throughput_pps() - 25.0).abs() < 1e-6);
}

#[test]
fn test_pipeline_stats_tracker_reset() {
    let mut tracker = PipelineStatsTracker::new(1.0);
    tracker.record_received(100);
    tracker.record_dropped(10);
    tracker.reset();
    assert_eq!(tracker.total_received(), 0);
    assert_eq!(tracker.total_dropped(), 0);
}

#[test]
fn test_pipeline_stats_tracker_window_accessor() {
    let tracker = PipelineStatsTracker::new(5.0);
    assert!((tracker.time_window_sec() - 5.0).abs() < 1e-6);
}
