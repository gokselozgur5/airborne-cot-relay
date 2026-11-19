use airborne_cot_relay::{downgrade_cot_xml, CotSchemaVersion};

#[test]
fn stripping_flow_tags_keeps_sibling_elements() {
    let xml = r#"<event version="2.0" uid="A"><detail><_flow-tags_ TAK-Server-1="t"></_flow-tags_><contact callsign="VIPER"/></detail></event>"#;
    let out = downgrade_cot_xml(xml, CotSchemaVersion::V1_0).unwrap();
    assert!(!out.contains("_flow-tags_"), "{out}");
    assert!(out.contains(r#"<contact callsign="VIPER"/>"#), "{out}");
}
