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
