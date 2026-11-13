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
