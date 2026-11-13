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

fn track(id: u64, lat_deg: f64) -> TrackSummary {
    TrackSummary {
        id,
        classification: "vehicle".into(),
        lat_deg,
        lon_deg: 139.0,
        hae_m: 0.0,
        ce90_m: 5.0,
        le90_m: 10.0,
        course_deg: 0.0,
        speed_mps: 0.0,
    }
}

fn queue_of(tracks: Vec<TrackSummary>) -> BoundedQueue<TrackSummary> {
    let mut queue = BoundedQueue::new(8, DropPolicy::DropOldest);
    tracks.into_iter().for_each(|t| drop(queue.push(t)));
    queue
}

fn render(t: &TrackSummary, now_ms: u64) -> String {
    format!("TRACK-{:04}@{now_ms}", t.id)
}

fn stepper(fail: bool) -> OutboundStep<ManualClock, Recorder> {
    let config = FilterConfig {
        stale_after_ms: 1_000,
        ..FilterConfig::default()
    };
    let mut filter = TrackFilter::new(config, ManualClock::new(0));
    assert!(filter.accept("TRACK-0099", 35.0, 139.0));
    filter.clock().set(5_000);
    let sink = Recorder {
        fail,
        ..Recorder::default()
    };
    OutboundStep::new(filter, sink, SendHealth::new(3))
}

#[test]
fn one_step_drains_queue_resets_gauge_and_evicts_stale() {
    let metrics = Metrics::new();
    metrics.set_queue_depth(2);
    let mut queue = queue_of(vec![track(1, 35.0), track(2, 35.1)]);
    let mut step = stepper(false);
    let report = step.step(5_000, &mut queue, &metrics, render);
    assert!(queue.is_empty());
    assert_eq!(metrics.snapshot(5_000).queue_depth, 0);
    assert_eq!(report.evicted, 1);
    assert_eq!(report.sent, 2);
    assert_eq!(step.filter().tracked_count(), 2);
    assert_eq!(step.sink().payloads, ["TRACK-0001@5000", "TRACK-0002@5000"]);
}

#[test]
fn duplicate_track_is_filtered_within_one_step() {
    let metrics = Metrics::new();
    let mut queue = queue_of(vec![track(1, 35.0), track(1, 35.0)]);
    let report = stepper(false).step(5_000, &mut queue, &metrics, render);
    assert_eq!((report.sent, report.filtered), (1, 1));
    assert_eq!(metrics.snapshot(5_000).sent, 1);
}

#[test]
fn failed_send_is_dropped_and_recorded_in_health() {
    let metrics = Metrics::new();
    let mut queue = queue_of(vec![track(1, 35.0), track(2, 35.1)]);
    let mut step = stepper(true);
    let report = step.step(5_000, &mut queue, &metrics, render);
    assert_eq!(report.failed, 2);
    assert_eq!(metrics.snapshot(5_000).dropped, 2);
    assert_eq!(step.health().failures(), (2, 2));
}
