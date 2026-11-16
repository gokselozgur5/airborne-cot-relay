use airborne_cot_relay::schema_version::{
    downgrade_cot_xml, CotSchemaVersion, DowngradeTransformer, SchemaNegotiator,
};

#[test]
fn test_integration_negotiate_and_transform_pipeline() {
    let negotiator = SchemaNegotiator::new(CotSchemaVersion::V2_0);
    let negotiated = negotiator.negotiate_from_header("1.0").unwrap();
    assert_eq!(negotiated, CotSchemaVersion::V1_0);

    let event_v2 = "<event version=\"2.0\" uid=\"UAV-1\"><point lat=\"37.5\"/><_flow-tags_ node=\"alpha\"/></event>";
    let transformer = DowngradeTransformer::new(negotiated);
    let output = transformer.transform(event_v2).unwrap();

    assert!(output.contains("version=\"1.0\""));
    assert!(!output.contains("_flow-tags_"));
}

#[test]
fn test_integration_legacy_unversioned_event_upgrade() {
    let unversioned = "<event uid=\"MISC-99\"><point lat=\"10.0\" lon=\"20.0\"/></event>";
    let ver = CotSchemaVersion::detect_from_xml(unversioned).unwrap();
    assert_eq!(ver, CotSchemaVersion::V1_0);

    let transformed = downgrade_cot_xml(unversioned, CotSchemaVersion::V1_0).unwrap();
    assert!(transformed.contains("version=\"1.0\""));
    assert!(transformed.contains("uid=\"MISC-99\""));
}
