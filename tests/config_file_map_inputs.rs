use airborne_cot_relay::config_file::map_inputs::InputKind;

#[test]
fn test_input_kind_variants() {
    let udp = InputKind::UdpTelemetry;
    let tcp = InputKind::TcpTelemetry;
    let det = InputKind::UdpDetections;

    assert_ne!(udp, tcp);
    assert_ne!(tcp, det);
    assert_eq!(udp, InputKind::UdpTelemetry);
}
