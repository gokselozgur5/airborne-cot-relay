use airborne_cot_relay::metrics::Metrics;
use std::sync::Arc;
use std::thread;

#[test]
fn concurrent_increments_are_not_lost() {
    let m = Arc::new(Metrics::new());
    let hs: Vec<_> = (0..8)
        .map(|_| {
            let m = Arc::clone(&m);
            thread::spawn(move || {
                for _ in 0..1000 {
                    m.add_packets_in(1);
                    m.add_dropped(2);
                }
            })
        })
        .collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    let s = m.snapshot(0);
    assert_eq!((s.packets_in, s.dropped), (8000, 16000));
}

#[test]
fn gauge_reflects_last_write() {
    let m = Arc::new(Metrics::new());
    let m2 = Arc::clone(&m);
    thread::spawn(move || m2.set_queue_depth(42))
        .join()
        .unwrap();
    assert_eq!(m.snapshot(0).queue_depth, 42);
    m.set_queue_depth(1);
    assert_eq!(m.snapshot(0).queue_depth, 1);
}

#[test]
fn concurrent_sends_are_counted() {
    let m = Arc::new(Metrics::new());
    let hs: Vec<_> = (0..4u64)
        .map(|i| {
            let m = Arc::clone(&m);
            thread::spawn(move || (0..500).for_each(|_| m.record_sent(i)))
        })
        .collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    assert_eq!(m.snapshot(10).sent, 2000);
    assert!(m.last_send_age_ms(10).is_some());
}
