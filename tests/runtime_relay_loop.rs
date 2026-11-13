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

fn frame(ts: u32) -> Vec<u8> {
    let t = NavTelemetry {
        timestamp_ms: ts,
        pos_ned_m: [0.0, 0.0, -100.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut b = [0u8; 64];
    let n = t.pack_uart(&mut b).unwrap();
    b[..n].to_vec()
}

fn detection(u: f64, v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

fn build_filter_step() -> OutboundStep<ManualClock, ScriptedSink> {
    let cfg = FilterConfig {
        min_interval_ms: 0,
        ..Default::default()
    };
    let filter = TrackFilter::new(cfg, ManualClock::new(1_000));
    OutboundStep::new(filter, ScriptedSink::default(), SendHealth::new(3))
}
