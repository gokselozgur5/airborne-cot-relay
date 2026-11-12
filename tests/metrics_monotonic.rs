use airborne_cot_relay::metrics::Metrics;

#[test]
fn out_of_order_record_does_not_move_last_send_back() {
    let m = Metrics::new();
    m.record_sent(200);
    m.record_sent(100);
    assert_eq!(m.last_send_age_ms(300), Some(100));
}

#[test]
fn concurrent_out_of_order_records_keep_max() {
    let m = std::sync::Arc::new(Metrics::new());
    let hs: Vec<_> = (0..4u64)
        .map(|t| {
            let m = m.clone();
            std::thread::spawn(move || {
                for i in 0..200u64 {
                    m.record_sent(if t == 0 { 1000 } else { i });
                }
            })
        })
        .collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    assert_eq!(m.last_send_age_ms(1000), Some(0));
}
