use airborne_cot_relay::pipeline::{step::OutboundStep, Pipeline};
use airborne_cot_relay::relay::{BoundedQueue, CotSink, DropPolicy, SendHealth};
use airborne_cot_relay::runtime::limit::RunLimit;
use airborne_cot_relay::runtime::relay_loop::RelayLoop;
use airborne_cot_relay::runtime::source::{DetectionSource, VecTelemetrySource};
use airborne_cot_relay::runtime::ShutdownSignal;
use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::{RelayConfig, RelayError};
struct Null;
impl CotSink for Null {
    fn send(&mut self, p: &[u8]) -> Result<usize, RelayError> {
        Ok(p.len())
    }
}
struct OneBad(bool);
impl DetectionSource for OneBad {
    fn poll_batch(
        &mut self,
    ) -> Result<Option<Vec<airborne_cot_relay::TargetDetection>>, RelayError> {
        if std::mem::take(&mut self.0) {
            Err(RelayError::DecodeError("malformed batch".into()))
        } else {
            Ok(None)
        }
    }
}
#[test]
fn droppable_error_does_not_stop_the_loop() {
    let outbound = OutboundStep::new(
        TrackFilter::new(FilterConfig::default(), ManualClock::new(1000)),
        Null,
        SendHealth::new(3),
    );
    let pipe = Pipeline::from_config(&RelayConfig::default()).unwrap();
    let queue = BoundedQueue::new(4, DropPolicy::DropOldest);
    let mut relay = RelayLoop::new(
        VecTelemetrySource::default(),
        OneBad(true),
        pipe,
        queue,
        outbound,
        ManualClock::new(1000),
        ShutdownSignal::new(),
        RunLimit::iterations(3),
        |_: &_, _: u64| String::new(),
    );
    assert!(relay.run().is_ok());
}
