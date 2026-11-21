use airborne_cot_relay::{CotStreamer, TrackSummary};
use sightline_cot::cot::CotEvent;

/// Uid of the first track reported by a freshly started relay at `lat_deg`.
fn first_track_uid(lat_deg: f64) -> String {
    let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:6969", 5000).unwrap();
    let (classification, lon_deg, hae_m) = ("vehicle".to_string(), 10.0, 0.0);
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
    let xml = streamer.build_cot_xml(&track, 1_731_369_600_000);
    CotEvent::from_xml(&xml).expect("valid CoT").uid
}

#[test]
fn two_relay_instances_do_not_share_event_uids() {
    // Two aircraft each report their first track, about 1200 km apart.
    assert_ne!(first_track_uid(37.0), first_track_uid(48.0));
}
