//! Repro-first integration test for wiring StaleGate into outbound CoT path (#413).

use airborne_cot_relay::pipeline::step::OutboundStep;
use airborne_cot_relay::relay::{BoundedQueue, CotSink, DropPolicy, SendHealth, TrackSummary};
use airborne_cot_relay::runtime::stale_gate::StaleGate;
use airborne_cot_relay::track_filter::{FilterConfig, ManualClock, TrackFilter};
use airborne_cot_relay::{PlatformPose, RelayError};

#[derive(Default)]
struct ScriptedSink(Vec<String>);

impl CotSink for ScriptedSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.0.push(String::from_utf8_lossy(payload).into_owned());
        Ok(payload.len())
    }
}


fn make_track(id: u64, lat_deg: f64) -> TrackSummary {
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

#[test]
fn test_stale_gate_withholds_cot_when_pose_is_stale_or_uninitialized() {
    let mut gate = StaleGate::new(1_000);
    assert!(!gate.is_active(1_000));

    let pose = PlatformPose {
        timestamp_ms: 1_000,
        lat_deg: 35.0,
        lon_deg: 139.0,
        hae_m: 100.0,
        roll_rad: 0.0,
        pitch_rad: 0.0,
        yaw_rad: 0.0,
    };
    gate.update(&pose);
    assert!(gate.is_active(1_500));
    assert!(!gate.is_active(2_500));
}

#[test]
fn test_outbound_step_gated_withholds_when_gate_is_paused() {
    let cfg = FilterConfig {
        min_interval_ms: 0,
        ..Default::default()
    };
    let filter = TrackFilter::new(cfg, ManualClock::new(2_500));
    let mut step = OutboundStep::new(filter, ScriptedSink::default(), SendHealth::new(3));
    let mut gate = StaleGate::new(1_000);
    gate.update(&PlatformPose {
        timestamp_ms: 1_000,
        lat_deg: 35.0,
        lon_deg: 139.0,
        hae_m: 100.0,
        roll_rad: 0.0,
        pitch_rad: 0.0,
        yaw_rad: 0.0,
    });

    let mut queue = BoundedQueue::new(4, DropPolicy::DropOldest);
    queue.push(make_track(1, 35.0));

    let report = step.step_gated(
        2_500,
        &mut queue,
        &airborne_cot_relay::metrics::Metrics::new(),
        Some(&gate),
        |t, now| format!("{t:?}@{now}"),
    );
    assert_eq!(report.sent, 0);
    assert_eq!(step.sink().0.len(), 0);

    gate.update(&PlatformPose {
        timestamp_ms: 2_500,
        lat_deg: 35.0,
        lon_deg: 139.0,
        hae_m: 100.0,
        roll_rad: 0.0,
        pitch_rad: 0.0,
        yaw_rad: 0.0,
    });
    queue.push(make_track(2, 35.1));
    let report_fresh = step.step_gated(
        2_500,
        &mut queue,
        &airborne_cot_relay::metrics::Metrics::new(),
        Some(&gate),
        |t, now| format!("{t:?}@{now}"),
    );
    assert_eq!(report_fresh.sent, 1);
    assert_eq!(step.sink().0.len(), 1);
}

