//! Concurrent multi-threaded test for lock-free CotRingBuffer.

use std::sync::Arc;
use std::thread;

use airborne_cot_relay::ring_buf::CotRingBuffer;
use sightline_cot::cot::CotEvent;

fn make_cot(i: usize) -> CotEvent {
    CotEvent {
        uid: format!("cot-{i}"),
        cot_type: "a-f-G-U-C".to_string(),
        how: "m-g".to_string(),
        time_ms: i as i64,
        stale_ms: i as i64 + 1000,
        lat_deg: 35.0,
        lon_deg: 42.0,
        hae_m: 100.0,
        ce_m: 5.0,
        le_m: 10.0,
        course_deg: 0.0,
        speed_mps: 10.0,
    }
}
