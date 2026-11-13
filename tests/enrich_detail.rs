use airborne_cot_relay::enrich::{CotDetail, DetailElement};

#[test]
fn test_cot_detail_escaping_and_order() {
    let mut detail = CotDetail::new();
    let contact = DetailElement::new("contact")
        .attr("callsign", "VIPER & \"GHOST\"")
        .attr("endpoint", "10.0.0.1:8080<test>");
    let remarks = DetailElement::new("remarks")
        .attr("source", "eo's sensor")
        .with_text("target <detected> & verified");

    detail.add_element(contact);
    detail.add_element(remarks);

    assert_eq!(detail.len(), 2);
    let xml = detail.render();
    let expected = "<detail><contact callsign=\"VIPER &amp; &quot;GHOST&quot;\" endpoint=\"10.0.0.1:8080&lt;test&gt;\"/><remarks source=\"eo&apos;s sensor\">target &lt;detected&gt; &amp; verified</remarks></detail>";
    assert_eq!(xml, expected);
    assert_eq!(format!("{detail}"), expected);
}
