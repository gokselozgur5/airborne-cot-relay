use airborne_cot_relay::enrich::{CallsignTemplate, CotDetail};

#[test]
fn test_callsign_template_default_formatting() {
    let template = CallsignTemplate::default();
    assert_eq!(template.format("HAWK", "vehicle", 42), "HAWK-vehicle-42");

    let contact = template.build_contact("HAWK", "vehicle", 42);
    assert_eq!(contact.tag(), "contact");
    assert_eq!(contact.attributes(), &[("callsign".to_string(), "HAWK-vehicle-42".to_string())]);

    let mut detail = CotDetail::new();
    detail.add_element(contact);
    assert_eq!(detail.render(), "<detail><contact callsign=\"HAWK-vehicle-42\"/></detail>");
}
