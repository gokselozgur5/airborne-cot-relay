use airborne_cot_relay::health::Heartbeat;
use airborne_cot_relay::xml_sanitize::sanitize_and_escape_attr;

#[test]
fn attribute_escaper_encodes_tab_and_line_breaks() {
    let escaped = sanitize_and_escape_attr("A\tB\nC\rD");
    assert_eq!(escaped, "A&#9;B&#10;C&#13;D");
}

#[test]
fn heartbeat_attributes_contain_only_xml_chars() {
    let hb = Heartbeat {
        uid: "relay\u{1}",
        status: "ok",
        time: "2026-01-01T00:00:00Z",
        stale: "2026-01-01T00:00:10Z",
    };
    let xml = hb.to_xml();
    assert!(xml.contains("uid=\"relay\""), "{xml:?}");
}
