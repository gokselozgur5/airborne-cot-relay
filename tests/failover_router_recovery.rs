use airborne_cot_relay::{FailoverRouter, NetworkInterface, RouteKind, RouterConfig};

#[test]
fn unhealthy_interface_is_offered_again_for_recovery() {
    let mut router = FailoverRouter::new(RouterConfig::default());
    let dest = "239.2.3.1:6969".parse().unwrap();
    let iface = NetworkInterface::new("radio0", "radio", RouteKind::Broadcast, dest);
    router.add_interface(iface);
    for _ in 0..3 {
        let decision = router.route(b"<event/>").unwrap();
        router.record_failure(&decision.interface_id);
    }
    let mut routed = 0;
    for _ in 0..1000 {
        if let Ok(decision) = router.route(b"<event/>") {
            router.record_success(&decision.interface_id, 8);
            routed += 1;
        }
    }
    assert!(
        routed > 0,
        "0 of 1000 packets routed after the link recovered"
    );
    assert_eq!(router.healthy_count(), 1);
}
