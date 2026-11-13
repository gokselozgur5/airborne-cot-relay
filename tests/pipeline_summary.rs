use airborne_cot_relay::pipeline::summary::track_summary;
use airborne_cot_relay::{GroundIntersection, TrackedObject};

fn track() -> TrackedObject {
    TrackedObject {
        id: 42,
        lat_deg: 35.1,
        lon_deg: 139.2,
        hae_m: 12.5,
        velocity_enu: [3.0, 4.0],
        speed_mps: 5.0,
        course_deg: 36.87,
        ce90_m: 4.25,
        classification: "vehicle".into(),
        hits: 3,
        is_confirmed: true,
    }
}
