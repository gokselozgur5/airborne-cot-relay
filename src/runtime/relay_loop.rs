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

impl<T, D, C: Clock, S: CotSink, R> RelayLoop<T, D, C, S, R> {
    /// Builds a new loop from sources, pipeline, queue, outbound step and controls.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        telemetry: T,
        detections: D,
        pipeline: Pipeline,
        queue: BoundedQueue<TrackSummary>,
        outbound: OutboundStep<C, S>,
        clock: C,
        shutdown: ShutdownSignal,
        limit: RunLimit,
        render: R,
    ) -> Self {
        Self {
            telemetry,
            detections,
            pipeline,
            queue,
            outbound,
            clock,
            shutdown,
            limit,
            render,
        }
    }
}

impl<T, D, C: Clock, S: CotSink, R> RelayLoop<T, D, C, S, R> {
    /// Telemetry source reference.
    pub fn telemetry(&self) -> &T {
        &self.telemetry
    }

    /// Mutable telemetry source reference.
    pub fn telemetry_mut(&mut self) -> &mut T {
        &mut self.telemetry
    }

    /// Detection source reference.
    pub fn detections(&self) -> &D {
        &self.detections
    }

    /// Mutable detection source reference.
    pub fn detections_mut(&mut self) -> &mut D {
        &mut self.detections
    }

    /// Pipeline reference.
    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }
}

impl<T, D, C: Clock, S: CotSink, R> RelayLoop<T, D, C, S, R> {
    /// Outbound step reference.
    pub fn outbound(&self) -> &OutboundStep<C, S> {
        &self.outbound
    }

    /// Queue reference.
    pub fn queue(&self) -> &BoundedQueue<TrackSummary> {
        &self.queue
    }

    /// Shutdown signal reference.
    pub fn shutdown(&self) -> &ShutdownSignal {
        &self.shutdown
    }

    /// Current run limit.
    pub fn limit(&self) -> RunLimit {
        self.limit
    }

    /// Updates the run limit.
    pub fn set_limit(&mut self, limit: RunLimit) {
        self.limit = limit;
    }
}

impl<T: TelemetrySource, D, C: Clock, S: CotSink, R> RelayLoop<T, D, C, S, R> {
    /// Ingests one chunk of telemetry if available.
    pub fn poll_telemetry(&mut self) -> Result<Option<usize>, RelayError> {
        match self.telemetry.poll_chunk()? {
            Some(chunk) => Ok(Some(self.pipeline.ingest_telemetry(&chunk).len())),
            None => Ok(None),
        }
    }
}
