use airborne_cot_relay::clock::{Clock, SystemClock};
use airborne_cot_relay::pipeline::{step::OutboundStep, Pipeline};
use airborne_cot_relay::relay::{BoundedQueue, CotSink, DropPolicy, SendHealth, TrackSummary};
use airborne_cot_relay::runtime::limit::RunLimit;
use airborne_cot_relay::runtime::relay_loop::RelayLoop;
use airborne_cot_relay::runtime::source::{TelemetrySource, VecDetectionSource};
use airborne_cot_relay::runtime::ShutdownSignal;
use airborne_cot_relay::track_filter::{FilterConfig, TrackFilter};
use airborne_cot_relay::{RelayConfig, RelayError};
use std::sync::Arc;
struct Null;
impl CotSink for Null {
    fn send(&mut self, p: &[u8]) -> Result<usize, RelayError> {
        Ok(p.len())
    }
}
struct CountingIdle(u64);
impl TelemetrySource for CountingIdle {
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        self.0 += 1;
        Ok(None)
    }
}
#[test]
fn idle_loop_does_not_spin() {
    let (clock, cfg) = (Arc::new(SystemClock::new()), FilterConfig::default());
    let (filter, q) = (
        TrackFilter::new(cfg, Arc::clone(&clock)),
        BoundedQueue::new(4, DropPolicy::DropOldest),
    );
    let outbound = OutboundStep::new(filter, Null, SendHealth::new(3));
    let pipeline = Pipeline::from_config(&RelayConfig::default()).unwrap();
    let limit = RunLimit::Deadline(clock.now_ms() as i64 + 200);
    let mut relay = RelayLoop::new(
        CountingIdle(0),
        VecDetectionSource::default(),
        pipeline,
        q,
        outbound,
        clock,
        ShutdownSignal::new(),
        limit,
        |_: &TrackSummary, _: u64| String::new(),
    );
    relay.run().unwrap();
    assert!(
        relay.telemetry().0 <= 2_000,
        "{} idle polls in 200 ms",
        relay.telemetry().0
    );
}
