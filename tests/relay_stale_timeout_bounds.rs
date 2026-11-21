use airborne_cot_relay::{CotStreamer, RelayConfig, TrackSummary};
use sightline_cot::cot::CotEvent;

/// True when the timeout is rejected by validation or yields a stale time after the event time.
fn is_safe(target_stale_timeout_ms: u64) -> bool {
    let config = RelayConfig {
        target_stale_timeout_ms,
        ..RelayConfig::default()
    };
    if config.validate().is_err() {
        return true;
    }
    let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:6969", target_stale_timeout_ms);
    let (classification, lat_deg, lon_deg, hae_m) = ("vehicle".to_string(), 37.0, -122.0, 0.0);
    let (ce90_m, le90_m, course_deg, speed_mps) = (5.0, 10.0, 0.0, 0.0);
    let track = TrackSummary {
        id: 1,
        classification,
        lat_deg,
        lon_deg,
        hae_m,
        ce90_m,
        le90_m,
        course_deg,
        speed_mps,
    };
    let xml = streamer.unwrap().build_cot_xml(&track, 1_731_369_600_000);
    let event = CotEvent::from_xml(&xml).expect("valid CoT");
    event.stale_ms > event.time_ms
}

#[test]
fn accepted_stale_timeout_never_yields_stale_before_time() {
    assert!(is_safe(u64::MAX));
}

#[test]
fn accepted_stale_timeout_does_not_overflow() {
    assert!(is_safe(i64::MAX as u64));
}
