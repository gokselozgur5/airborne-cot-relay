use airborne_cot_relay::metrics::Metrics;

#[test]
fn fresh_snapshot_format_is_pinned() {
    let out = Metrics::new().snapshot(10).render();
    let want = "packets_in 0\ndropped 0\nsent 0\nreconnects 0\n\
queue_depth 0\nlast_send_age_ms never\n";
    assert_eq!(out, want);
}

#[test]
fn populated_snapshot_format_is_pinned() {
    let m = Metrics::new();
    m.add_packets_in(7);
    m.add_dropped(2);
    m.add_reconnect();
    m.record_sent(100);
    m.set_queue_depth(4);
    let want = "packets_in 7\ndropped 2\nsent 1\nreconnects 1\n\
queue_depth 4\nlast_send_age_ms 25\n";
    assert_eq!(m.snapshot(125).render(), want);
}

#[test]
fn render_has_six_lines_and_trailing_newline() {
    let out = Metrics::new().snapshot(0).render();
    assert!(out.ends_with('\n'));
    assert_eq!(out.lines().count(), 6);
    assert!(out.lines().all(|l| l.split(' ').count() == 2));
}
