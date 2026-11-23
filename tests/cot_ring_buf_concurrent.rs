//! Concurrent multi-threaded test for lock-free CotRingBuffer.

use std::sync::atomic::{AtomicBool, Ordering};
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
        start_ms: i as i64,
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

#[test]
fn test_cot_ring_buffer_concurrent_spsc() {
    let buf = Arc::new(CotRingBuffer::new(64));
    let buf_producer = Arc::clone(&buf);
    let buf_consumer = Arc::clone(&buf);
    let n = 200;

    let running = Arc::new(AtomicBool::new(true));
    let consumer_running = Arc::clone(&running);

    let consumer = thread::spawn(move || {
        let mut received = Vec::new();
        while consumer_running.load(Ordering::Relaxed) {
            if let Some(event) = buf_consumer.pop() {
                received.push(event);
            } else {
                thread::yield_now();
            }
        }
        while let Some(event) = buf_consumer.pop() {
            received.push(event);
        }
        received
    });

    let producer = thread::spawn(move || {
        for i in 0..n {
            buf_producer.push_overwrite(make_cot(i));
        }
    });

    producer.join().expect("producer finished");
    running.store(false, Ordering::Relaxed);
    let mut consumed = consumer.join().expect("consumer finished");

    let mut remaining = buf.drain_all();
    assert!(remaining.len() <= 64);
    consumed.append(&mut remaining);

    let drops = buf.dropped_count();
    assert_eq!(consumed.len() + drops, n);
    assert!(!consumed.is_empty());

    let mut uids = std::collections::HashSet::new();
    for event in &consumed {
        assert!(
            uids.insert(event.uid.clone()),
            "duplicate event received: {}",
            event.uid
        );
    }
}
