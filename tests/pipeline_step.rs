use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::pipeline::step::OutboundStep;
use airborne_cot_relay::relay::{BoundedQueue, CotSink, DropPolicy, SendHealth};
use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::{RelayError, TrackSummary};

#[derive(Default)]
struct Recorder {
    payloads: Vec<String>,
    fail: bool,
}

impl CotSink for Recorder {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        if self.fail {
            return Err(RelayError::IoError("refused".into()));
        }
        self.payloads
            .push(String::from_utf8_lossy(payload).into_owned());
        Ok(payload.len())
    }
}
