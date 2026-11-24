//! One pipeline tick: drain the outbound queue, send, evict stale filter entries.

/// Counts produced by one `OutboundStep::step` call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StepReport {
    /// Tracks that reached the sink.
    pub sent: usize,
    /// Tracks the sink rejected or wrote short.
    pub failed: usize,
    /// Tracks held back by the rate and movement filter.
    pub filtered: usize,
    /// Filter entries evicted as stale after the drain.
    pub evicted: usize,
}

impl StepReport {
    /// Returns the sum of sent, failed, and filtered tracks.
    pub fn total_processed(&self) -> usize {
        self.sent + self.failed + self.filtered
    }

    /// Returns true if no tracks were processed or evicted.
    pub fn is_empty(&self) -> bool {
        self.total_processed() == 0 && self.evicted == 0
    }
}

impl std::ops::Add for StepReport {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            sent: self.sent + rhs.sent,
            failed: self.failed + rhs.failed,
            filtered: self.filtered + rhs.filtered,
            evicted: self.evicted + rhs.evicted,
        }
    }
}

impl std::ops::AddAssign for StepReport {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

use crate::relay::{CotSink, SendHealth};
use crate::track_filter::{Clock, TrackFilter};

/// Output side of the pipeline: rate filter, sink and send health.
#[derive(Debug)]
pub struct OutboundStep<C: Clock, S: CotSink> {
    filter: TrackFilter<C>,
    sink: S,
    health: SendHealth,
}

impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// Builds the step from its filter, sink and send health tracker.
    pub fn new(filter: TrackFilter<C>, sink: S, health: SendHealth) -> Self {
        Self {
            filter,
            sink,
            health,
        }
    }
}

impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// The rate and movement filter.
    pub fn filter(&self) -> &TrackFilter<C> {
        &self.filter
    }

    /// The sink tracks are sent to.
    pub fn sink(&self) -> &S {
        &self.sink
    }

    /// Consecutive and total send failures.
    pub fn health(&self) -> &SendHealth {
        &self.health
    }

    /// Mutable reference to the rate and movement filter.
    pub fn filter_mut(&mut self) -> &mut TrackFilter<C> {
        &mut self.filter
    }

    /// Mutable reference to the sink tracks are sent to.
    pub fn sink_mut(&mut self) -> &mut S {
        &mut self.sink
    }

    /// Mutable reference to send health tracker.
    pub fn health_mut(&mut self) -> &mut SendHealth {
        &mut self.health
    }

    /// Decomposes the step into filter, sink, and health tracker.
    pub fn into_parts(self) -> (TrackFilter<C>, S, SendHealth) {
        (self.filter, self.sink, self.health)
    }
}

use crate::metrics::Metrics;
use crate::relay::{send_tracked, TrackSummary};

/// Filters, encodes and sends one track, updating `health` and `metrics`.
pub(crate) fn send_filtered_track<C: Clock, S: CotSink, F>(
    filter: &mut TrackFilter<C>,
    sink: &mut S,
    health: &mut SendHealth,
    metrics: &Metrics,
    track: &TrackSummary,
    now_ms: u64,
    render: F,
) -> Option<bool>
where
    F: FnOnce(&TrackSummary, u64) -> String,
{
    let uid = format!("TRACK-{:04}", track.id);
    if !filter.accept(&uid, track.lat_deg, track.lon_deg) {
        return None;
    }
    let ok = send_tracked(sink, health, render(track, now_ms).as_bytes()).is_ok();
    if ok {
        metrics.record_sent(now_ms);
    } else {
        metrics.add_dropped(1);
    }
    Some(ok)
}

impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// Filters, renders and sends one track; `None` means the filter held it back.
    fn send_one<F>(
        &mut self,
        track: &TrackSummary,
        now_ms: u64,
        m: &Metrics,
        r: &mut F,
    ) -> Option<bool>
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        send_filtered_track(
            &mut self.filter,
            &mut self.sink,
            &mut self.health,
            m,
            track,
            now_ms,
            r,
        )
    }
}

use crate::relay::BoundedQueue;

impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// Drains `queue` through `render` to the sink, evicts stale entries, sets the gauge.
    pub fn step<F>(
        &mut self,
        now_ms: u64,
        queue: &mut BoundedQueue<TrackSummary>,
        metrics: &Metrics,
        render: F,
    ) -> StepReport
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        self.step_gated(now_ms, queue, metrics, None, render)
    }

    /// Drains `queue` through `render` to the sink with an optional `StaleGate`.
    pub fn step_gated<F>(
        &mut self,
        now_ms: u64,
        queue: &mut BoundedQueue<TrackSummary>,
        metrics: &Metrics,
        gate: Option<&crate::runtime::stale_gate::StaleGate>,
        mut render: F,
    ) -> StepReport
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        let mut report = StepReport::default();
        let is_blocked = gate.is_some_and(|g| !g.is_active(now_ms));
        while let Some(track) = queue.pop() {
            if is_blocked {
                report.filtered += 1;
                metrics.add_dropped(1);
                continue;
            }
            match self.send_one(&track, now_ms, metrics, &mut render) {
                None => report.filtered += 1,
                Some(true) => report.sent += 1,
                Some(false) => report.failed += 1,
            }
        }
        report.evicted = self.filter.evict_stale();
        metrics.set_queue_depth(queue.len() as u64);
        report
    }

    /// Drains `stage` through `render` to the sink, evicts stale entries, updates gauge.
    pub fn step_stage<F>(
        &mut self,
        now_ms: u64,
        stage: &mut crate::pipeline::queue_stage::QueueStage<TrackSummary>,
        metrics: &Metrics,
        render: F,
    ) -> StepReport
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        self.step_stage_gated(now_ms, stage, metrics, None, render)
    }

    /// Drains `stage` through `render` to the sink with an optional `StaleGate`.
    pub fn step_stage_gated<F>(
        &mut self,
        now_ms: u64,
        stage: &mut crate::pipeline::queue_stage::QueueStage<TrackSummary>,
        metrics: &Metrics,
        gate: Option<&crate::runtime::stale_gate::StaleGate>,
        mut render: F,
    ) -> StepReport
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        let mut report = StepReport::default();
        let is_blocked = gate.is_some_and(|g| !g.is_active(now_ms));
        while let Some(track) = stage.pop(metrics) {
            if is_blocked {
                report.filtered += 1;
                metrics.add_dropped(1);
                continue;
            }
            match self.send_one(&track, now_ms, metrics, &mut render) {
                None => report.filtered += 1,
                Some(true) => report.sent += 1,
                Some(false) => report.failed += 1,
            }
        }
        report.evicted = self.filter.evict_stale();
        report
    }
}

use super::Pipeline;

impl Pipeline {
    /// Steps `outbound`, draining `queue` with pipeline metrics and `render`.
    pub fn step<C: Clock, S: CotSink, F>(
        &self,
        outbound: &mut OutboundStep<C, S>,
        now_ms: u64,
        queue: &mut BoundedQueue<TrackSummary>,
        render: F,
    ) -> StepReport
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        outbound.step_gated(
            now_ms,
            queue,
            &self.metrics,
            Some(self.stale_gate()),
            render,
        )
    }

    /// Steps `outbound` without telemetry staleness gating.
    pub fn step_ungated<C: Clock, S: CotSink, F>(
        &self,
        outbound: &mut OutboundStep<C, S>,
        now_ms: u64,
        queue: &mut BoundedQueue<TrackSummary>,
        render: F,
    ) -> StepReport
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        outbound.step(now_ms, queue, &self.metrics, render)
    }
}
