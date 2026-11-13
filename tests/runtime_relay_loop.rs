//! Integration tests for RelayLoop.

use airborne_cot_relay::pipeline::step::OutboundStep;
use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::relay::{BoundedQueue, CotSink, DropPolicy, SendHealth, TrackSummary};
use airborne_cot_relay::runtime::limit::RunLimit;
use airborne_cot_relay::runtime::relay_loop::RelayLoop;
use airborne_cot_relay::runtime::source::{
    TelemetrySource, VecDetectionSource, VecTelemetrySource,
};
use airborne_cot_relay::runtime::ShutdownSignal;
use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::{RelayConfig, RelayError, TargetDetection};
use stm32_navcore::protocol::NavTelemetry;

#[derive(Default)]
struct ScriptedSink(Vec<String>);

impl CotSink for ScriptedSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.0.push(String::from_utf8_lossy(payload).into_owned());
        Ok(payload.len())
    }
}
