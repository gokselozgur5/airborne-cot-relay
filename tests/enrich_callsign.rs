use airborne_cot_relay::enrich::{CallsignTemplate, CallsignTemplateError, DetailElement};

#[test]
fn test_callsign_template_default_formatting() {
    let template = CallsignTemplate::default();
    assert_eq!(template.format("HAWK", "vehicle", 42), "HAWK-vehicle-42");

    let contact = template.build_contact("HAWK", "vehicle", 42);
    assert_eq!(contact.tag(), "contact");
    assert_eq!(
        contact.attributes(),
        &[("callsign".to_string(), "HAWK-vehicle-42".to_string())]
    );
    assert_eq!(contact.to_xml(), "<contact callsign=\"HAWK-vehicle-42\"/>");
}

#[test]
fn test_callsign_template_validation_errors() {
    let err = CallsignTemplate::parse("{prefix}-{unknown}-{id}").unwrap_err();
    assert_eq!(
        err,
        CallsignTemplateError::UnknownPlaceholder("unknown".to_string())
    );

    let unclosed = CallsignTemplate::parse("{prefix}-{class").unwrap_err();
    assert_eq!(unclosed, CallsignTemplateError::UnclosedBrace);

    let custom = CallsignTemplate::parse("VIPER_{class}_{id}").unwrap();
    assert_eq!(custom.format("UNUSED", "air", 7), "VIPER_air_7");
}
