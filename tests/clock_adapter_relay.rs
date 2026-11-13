use airborne_cot_relay::clock::adapters::RelayClockAdapter;
use airborne_cot_relay::clock::ManualClock;
use airborne_cot_relay::relay::{self, Clock as _, CotStreamer, TrackSummary};

fn track() -> TrackSummary {
    TrackSummary {
        id: 7,
        classification: "person".to_string(),
        lat_deg: 36.0,
        lon_deg: 32.0,
        hae_m: 10.0,
        ce90_m: 2.0,
        le90_m: 5.0,
        course_deg: 0.0,
        speed_mps: 1.0,
    }
}

#[test]
fn adapted_clock_renders_same_time_as_native_clock() {
    let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:6969", 5000).unwrap();
    let unified = ManualClock::new(1_731_369_600_000);
    unified.advance_ms(1_000);
    let native = relay::ManualClock::new(1_731_369_601_000);
    let adapted = streamer.build_cot_xml_now(&track(), &RelayClockAdapter(&unified));
    assert_eq!(adapted, streamer.build_cot_xml_now(&track(), &native));
    assert!(adapted.contains(r#"time="2024-11-12T00:00:01"#));
}

#[test]
fn adapter_saturates_above_i64_max() {
    let unified = ManualClock::new(u64::MAX);
    assert_eq!(RelayClockAdapter(&unified).now_ms(), i64::MAX);
    unified.set_ms(42);
    assert_eq!(RelayClockAdapter(&unified).now_ms(), 42);
}
