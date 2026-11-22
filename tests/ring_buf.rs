//! Tests for ring buffer push_overwrite drop counter semantics.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use airborne_cot_relay::ring_buf::CotRingBuffer;
use sightline_cot::cot::CotEvent;

fn make_cot(uid: &str) -> CotEvent {
    CotEvent {
        uid: uid.to_string(),
        cot_type: "a-f-G-U-C".to_string(),
        how: "m-g".to_string(),
        time_ms: 1000,
        stale_ms: 2000,
        lat_deg: 35.0,
        lon_deg: 42.0,
        hae_m: 100.0,
        ce_m: 5.0,
        le_m: 10.0,
        course_deg: 0.0,
        speed_mps: 10.0,
    }
}

#[test]
fn test_push_overwrite_no_spurious_drop_on_concurrent_drain() {
    let mut detected_spurious_drop = false;

    for _ in 0..200 {
        let buf = Arc::new(CotRingBuffer::new(2));
        buf.push(make_cot("cot-1")).expect("push 1");
        buf.push(make_cot("cot-2")).expect("push 2");

        let running = Arc::new(AtomicBool::new(true));
        let consumer_buf = Arc::clone(&buf);
        let consumer_running = Arc::clone(&running);

        let consumer = thread::spawn(move || {
            let mut consumed = 0;
            while consumer_running.load(Ordering::Relaxed) {
                if consumer_buf.pop().is_some() {
                    consumed += 1;
                }
                thread::yield_now();
            }
            while consumer_buf.pop().is_some() {
                consumed += 1;
            }
            consumed
        });

        for i in 0..500 {
            buf.push_overwrite(make_cot(&format!("cot-p-{i}")));
        }

        running.store(false, Ordering::Relaxed);
        let total_consumed = consumer.join().expect("consumer join");
        let remaining = buf.drain_all().len();
        let drops = buf.dropped_count();

        if total_consumed + remaining + drops != 2 + 500 {
            detected_spurious_drop = true;
            break;
        }
    }

    assert!(
        !detected_spurious_drop,
        "concurrent consumer must not cause push_overwrite to record spurious drops"
    );
}
