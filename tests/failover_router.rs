use airborne_cot_relay::{
    FailoverPolicy, FailoverRouter, NetworkInterface, RouteKind, RouterConfig,
};

#[test]
fn disabled_interface_does_not_trigger_failover_status() {
    let mut router = FailoverRouter::new(RouterConfig {
        policy: FailoverPolicy::PriorityPrimary,
        max_payload_size: 65507,
    });
    let dest_a = "192.168.1.10:6969".parse().unwrap();
    let dest_b = "192.168.1.11:6969".parse().unwrap();

    let iface_a =
        NetworkInterface::new("iface-a", "eth0", RouteKind::Unicast, dest_a).with_priority(1);
    let iface_b =
        NetworkInterface::new("iface-b", "eth1", RouteKind::Unicast, dest_b).with_priority(5);

    router.add_interface(iface_a);
    router.add_interface(iface_b);

    router.set_enabled("iface-a", false);

    let decision = router.route(b"<event/>").expect("route should succeed");
    assert_eq!(decision.interface_id, "iface-b");
    assert!(
        !decision.is_failover,
        "routing to highest priority enabled interface should not be flagged as failover"
    );
    assert_eq!(
        router.stats().failover_events,
        0,
        "failover_events must remain 0 when primary is disabled"
    );
}
