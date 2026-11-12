use airborne_cot_relay::metrics::Metrics;

#[test]
fn fresh_snapshot_format_is_pinned() {
    let out = Metrics::new().snapshot(10).render();
    let want = "packets_in 0\ndropped 0\nsent 0\nreconnects 0\n\
queue_depth 0\nlast_send_age_ms never\n";
    assert_eq!(out, want);
}
