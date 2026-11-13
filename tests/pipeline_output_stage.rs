use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::pipeline::output_stage::OutputStage;
use airborne_cot_relay::relay::{CotSink, SendHealth};
use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::{RelayError, TrackSummary};

/// Sink that fails while `fail` is set and records delivered payloads.
#[derive(Default)]
struct ScriptedSink {
    fail: bool,
    payloads: Vec<String>,
}

impl CotSink for ScriptedSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        if self.fail {
            return Err(RelayError::IoError("scripted failure".into()));
        }
        self.payloads
            .push(String::from_utf8_lossy(payload).into_owned());
        Ok(payload.len())
    }
}

fn stage() -> OutputStage<ManualClock> {
    let filter = TrackFilter::new(FilterConfig::default(), ManualClock::new(5_000));
    let encode = Box::new(|t: &TrackSummary, now_ms: u64| format!("{}@{now_ms}", t.id));
    OutputStage::new(filter, SendHealth::new(3), encode)
}

fn track(id: u64) -> TrackSummary {
    TrackSummary {
        id,
        classification: "vehicle".into(),
        lat_deg: 35.0 + id as f64 * 0.001,
        lon_deg: 139.0,
        hae_m: 0.0,
        ce90_m: 5.0,
        le90_m: 3.0,
        course_deg: 0.0,
        speed_mps: 0.0,
    }
}

#[test]
fn accepted_tracks_are_sent_and_counted() {
    let (mut stage, mut sink, metrics) = (stage(), ScriptedSink::default(), Metrics::new());
    assert_eq!(stage.emit(&mut sink, &metrics, &[track(1), track(2)]), 2);
    assert_eq!(sink.payloads, vec!["1@5000", "2@5000"]);
    assert_eq!(metrics.snapshot(5_000).sent, 2);
    assert_eq!(stage.health().failures(), (0, 0));
}

#[test]
fn filtered_tracks_never_reach_the_sink() {
    let (mut stage, mut sink, metrics) = (stage(), ScriptedSink::default(), Metrics::new());
    assert_eq!(stage.emit(&mut sink, &metrics, &[track(1), track(1)]), 1);
    assert_eq!(stage.emit(&mut sink, &metrics, &[track(1)]), 0);
    assert_eq!(sink.payloads.len(), 1);
    assert_eq!(metrics.snapshot(5_000).sent, 1);
}
