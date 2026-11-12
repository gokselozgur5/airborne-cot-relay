use airborne_cot_relay::metrics::Metrics;
use std::sync::Arc;

#[test]
fn sent_never_pairs_with_never_sent() {
    for _ in 0..2000 {
        let m = Arc::new(Metrics::new());
        let w = {
            let m = m.clone();
            std::thread::spawn(move || m.record_sent(5))
        };
        for _ in 0..200 {
            let s = m.snapshot(10);
            assert!(s.sent == 0 || s.last_send_age_ms.is_some(), "{s:?}");
        }
        w.join().unwrap();
    }
}
