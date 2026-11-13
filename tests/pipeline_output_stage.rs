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
