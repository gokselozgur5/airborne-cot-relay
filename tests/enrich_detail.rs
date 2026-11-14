use airborne_cot_relay::output::enrich::CotDetail;

#[test]
fn test_cot_detail_self_closing_with_attributes() {
    let d = CotDetail::new("contact")
        .with_attr("callsign", "VIPER-1")
        .with_attr("endpoint", "10.0.0.1:4242");
    assert_eq!(d.to_xml(), "<contact callsign=\"VIPER-1\" endpoint=\"10.0.0.1:4242\"/>");
}

#[test]
fn test_cot_detail_with_text_and_escaping() {
    let d = CotDetail::new("remarks")
        .with_attr("source", "sensor<1>")
        .with_text("target & priority \"high\"");
    assert_eq!(
        d.to_xml(),
        "<remarks source=\"sensor&lt;1&gt;\">target &amp; priority &quot;high&quot;</remarks>"
    );
}
