use airborne_cot_relay::health::Heartbeat;

fn hb<'a>(uid: &'a str, status: &'a str) -> Heartbeat<'a> {
    Heartbeat {
        uid,
        status,
        time: "2026-01-01T00:00:00Z",
        stale: "2026-01-01T00:00:30Z",
    }
}

#[test]
fn renders_event_with_status() {
    let x = hb("relay-1", "Healthy").to_xml();
    assert!(x.starts_with("<event version=\"2.0\" uid=\"relay-1\""));
    assert!(x.contains("<status relay=\"Healthy\"/>"));
    assert!(x.contains("stale=\"2026-01-01T00:00:30Z\""));
    assert!(x.ends_with("</event>"));
}
