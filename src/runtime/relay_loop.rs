//! Continuous loop polling inputs, updating the pipeline, and stepping outbound sinks.

use crate::clock::adapters::RelayClockAdapter;
use crate::config::RelayError;
use crate::pipeline::step::{OutboundStep, StepReport};
use crate::pipeline::Pipeline;
use crate::relay::{BoundedQueue, CotSink, TrackSummary};
use crate::runtime::limit::RunLimit;
use crate::runtime::source::{DetectionSource, TelemetrySource};
use crate::runtime::ShutdownSignal;
use crate::track_filter::Clock;

/// Drives telemetry and detection sources through the pipeline to outbound sinks.
pub struct RelayLoop<T, D, C: Clock, S: CotSink, R> {
    telemetry: T,
    detections: D,
    pipeline: Pipeline,
    queue: BoundedQueue<TrackSummary>,
    outbound: OutboundStep<C, S>,
    clock: C,
    shutdown: ShutdownSignal,
    limit: RunLimit,
    render: R,
}
