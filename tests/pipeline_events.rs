use airborne_cot_relay::pipeline::events::{EventTracker, PipelineEvent};
use airborne_cot_relay::RelayError;

#[test]
fn three_failures_emit_one_degraded_event() {
    let mut tracker = EventTracker::new(3);
    for _ in 0..2 {
        tracker.record_send(false);
    }
    assert!(tracker.drain().is_empty());
    for _ in 0..4 {
        tracker.record_send(false);
    }
    let events = tracker.drain();
    assert_eq!(events, [PipelineEvent::SinkDegraded { failures: 3 }]);
}

#[test]
fn recovery_emits_one_recovered_event() {
    let mut tracker = EventTracker::new(3);
    (0..3).for_each(|_| tracker.record_send(false));
    tracker.drain();
    (0..3).for_each(|_| tracker.record_send(true));
    assert_eq!(tracker.drain(), [PipelineEvent::SinkRecovered]);
}
