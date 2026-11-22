use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::pipeline::output_stage::OutputStage;
use airborne_cot_relay::relay::{CotSink, SendHealth};
use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::{RelayError, TrackSummary};

struct FailingSink;

impl CotSink for FailingSink {
    fn send(&mut self, _bytes: &[u8]) -> Result<usize, RelayError> {
        Err(RelayError::IoError("send error".into()))
    }
}

fn dummy_track() -> TrackSummary {
    TrackSummary {
        id: 1,
        classification: "test".into(),
        lat_deg: 0.0,
        lon_deg: 0.0,
        hae_m: 0.0,
        ce90_m: 0.0,
        le90_m: 0.0,
        course_deg: 0.0,
        speed_mps: 0.0,
    }
}

#[test]
fn test_output_stage_failed_send_records_dropped() {
    let filter = TrackFilter::new(FilterConfig::default(), ManualClock::new(0));
    let mut stage = OutputStage::new(filter, SendHealth::new(3), Box::new(|_, _| "cot".into()));
    let mut sink = FailingSink;
    let metrics = Metrics::new();
    let sent = stage.send_one(&mut sink, &metrics, &dummy_track());
    assert!(!sent);
    let snapshot = metrics.snapshot(0);
    assert_eq!(snapshot.dropped, 1);
}
