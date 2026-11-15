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

#[test]
fn success_before_threshold_emits_nothing() {
    let mut tracker = EventTracker::new(3);
    [false, false, true, false, false]
        .into_iter()
        .for_each(|ok| tracker.record_send(ok));
    assert!(tracker.drain().is_empty());
}

#[test]
fn stale_telemetry_emits_once_until_fresh_again() {
    let mut tracker = EventTracker::new(3);
    [500, 2_500, 4_000, 100, 3_000]
        .into_iter()
        .for_each(|age| tracker.record_telemetry_age(age, 2_000));
    let events = tracker.drain();
    let expected = [2_500, 3_000].map(|age_ms| PipelineEvent::TelemetryStale { age_ms });
    assert_eq!(events, expected);
}

#[test]
fn decode_error_carries_the_error_text() {
    let mut tracker = EventTracker::new(3);
    tracker.record_decode_error(&RelayError::DecodeError("bad crc".into()));
    let events = tracker.drain();
    assert!(
        matches!(&events[..], [PipelineEvent::DecodeError { message }] if message.contains("bad crc"))
    );
}

#[test]
fn pipeline_event_display_formats_all_variants() {
    let degraded = PipelineEvent::SinkDegraded { failures: 5 };
    assert_eq!(degraded.to_string(), "sink degraded after 5 consecutive failures");
    let recovered = PipelineEvent::SinkRecovered;
    assert_eq!(recovered.to_string(), "sink recovered");
    let stale = PipelineEvent::TelemetryStale { age_ms: 1200 };
    assert_eq!(stale.to_string(), "telemetry stale: age 1200 ms");
    let decode = PipelineEvent::DecodeError { message: "corrupt".into() };
    assert_eq!(decode.to_string(), "decode error: corrupt");
}

#[test]
fn event_tracker_state_accessors_reflect_conditions() {
    let mut tracker = EventTracker::new(2);
    assert_eq!(tracker.degrade_after(), 2);
    assert_eq!(tracker.consecutive_failures(), 0);
    assert!(!tracker.is_degraded());
    assert!(!tracker.is_telemetry_stale());
    tracker.record_send(false);
    assert_eq!(tracker.consecutive_failures(), 1);
    assert!(!tracker.is_degraded());
    tracker.record_send(false);
    assert_eq!(tracker.consecutive_failures(), 2);
    assert!(tracker.is_degraded());
    tracker.record_telemetry_age(3000, 1000);
    assert!(tracker.is_telemetry_stale());
}

#[test]
fn event_tracker_pending_inspection_and_reset() {
    let mut tracker = EventTracker::new(1);
    assert!(tracker.is_empty());
    assert_eq!(tracker.pending_count(), 0);
    assert!(tracker.pending().is_empty());
    tracker.record_send(false);
    assert!(!tracker.is_empty());
    assert_eq!(tracker.pending_count(), 1);
    assert_eq!(tracker.pending()[0], PipelineEvent::SinkDegraded { failures: 1 });
    tracker.reset();
    assert!(tracker.is_empty());
    assert_eq!(tracker.pending_count(), 0);
    assert_eq!(tracker.consecutive_failures(), 0);
    assert!(!tracker.is_degraded());
}
