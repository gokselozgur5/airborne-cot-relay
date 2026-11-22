use airborne_cot_relay::app::exit::status_for;
use airborne_cot_relay::clock::{self, Clock};
use airborne_cot_relay::error::exit::exit_code;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::pipeline::{output_stage::OutputStage, step::OutboundStep};
use airborne_cot_relay::relay::{self, BoundedQueue, CotSink, DropPolicy, SendHealth};
use airborne_cot_relay::track_filter::{self, FilterConfig, TrackFilter};
use airborne_cot_relay::{RelayError, TrackSummary};
use std::{any::TypeId, mem::size_of, thread::sleep, time::Duration};

struct FailSink;
impl CotSink for FailSink {
    fn send(&mut self, _: &[u8]) -> Result<usize, RelayError> {
        Err(RelayError::IoError("send error".into()))
    }
}

fn tid<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

#[test]
fn clock_send_and_exit_helpers_are_unified() {
    let trk = TrackSummary {
        id: 1,
        classification: "v".into(),
        lat_deg: 0.0,
        lon_deg: 0.0,
        hae_m: 0.0,
        ce90_m: 1.0,
        le90_m: 1.0,
        course_deg: 0.0,
        speed_mps: 0.0,
    };
    let (sm, pm) = (Metrics::new(), Metrics::new());
    let sf = TrackFilter::new(FilterConfig::default(), track_filter::ManualClock::new(0));
    let mut stage = OutputStage::new(sf, SendHealth::new(3), Box::new(|_, _| "cot".into()));
    assert!(!stage.send_one(&mut FailSink, &sm, &trk));
    let pf = TrackFilter::new(FilterConfig::default(), track_filter::ManualClock::new(0));
    let mut step = OutboundStep::new(pf, FailSink, SendHealth::new(3));
    let mut q = BoundedQueue::new(4, DropPolicy::DropOldest);
    q.push(trk);
    assert_eq!(step.step(0, &mut q, &pm, |_, _| "cot".into()).failed, 1);
    assert_eq!((sm.snapshot(0).dropped, pm.snapshot(0).dropped), (1, 1));
    assert_eq!(tid::<clock::ManualClock>(), tid::<relay::ManualClock>());
    assert_eq!(
        tid::<clock::ManualClock>(),
        tid::<track_filter::ManualClock>()
    );
    assert_eq!(tid::<clock::SystemClock>(), tid::<relay::SystemClock>());
    assert_eq!(size_of::<clock::SystemClock>(), 0);
    let t0 = clock::SystemClock::new().now_ms();
    sleep(Duration::from_millis(5));
    assert!(clock::SystemClock::default().now_ms() >= t0 + 2);
    for e in [
        RelayError::InvalidConfig("c".into()),
        RelayError::IoError("i".into()),
        RelayError::DecodeError("d".into()),
        RelayError::TargetingError("t".into()),
    ] {
        assert_eq!(i32::from(status_for(&e)), exit_code(&e));
    }
}

#[test]
fn unit_system_clock_and_manual_clock_aliases_work() {
    let manual = clock::ManualClock::new(100);
    manual.advance(50);
    assert_eq!(manual.now_ms(), 150);
    assert_eq!(relay::Clock::now_ms(&manual), 150);
    manual.set(42);
    assert_eq!(manual.now_ms(), 42);
    let unit_now = clock::SystemClock.now_ms();
    assert!(unit_now >= 1_700_000_000_000);
    assert!(relay::Clock::now_ms(&clock::SystemClock) >= 1_700_000_000_000);
}
