//! Multi-interface IP broadcast and unicast failover router.

use std::net::SocketAddr;

/// Routing target transport kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteKind {
    Broadcast,
    Unicast,
}

/// Errors occurring during failover routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    EmptyPayload,
    PayloadTooLarge { size: usize, limit: usize },
    NoHealthyInterface,
    InterfaceNotFound(String),
}

/// Failover routing policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverPolicy {
    PriorityPrimary,
    RoundRobin,
    BroadcastWithUnicastBackup,
}

/// Statistics for failover router operation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RouterStats {
    pub packets_received: u64,
    pub packets_routed: u64,
    pub packets_dropped: u64,
    pub broadcast_dispatched: u64,
    pub unicast_dispatched: u64,
    pub failover_events: u64,
    pub total_bytes_routed: u64,
}

/// Network interface configuration and telemetry state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterface {
    pub id: String,
    pub name: String,
    pub kind: RouteKind,
    pub destination: SocketAddr,
    pub priority: u32,
    pub enabled: bool,
    pub healthy: bool,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub bytes_sent: u64,
    pub packets_sent: u64,
}

impl NetworkInterface {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: RouteKind,
        destination: SocketAddr,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            destination,
            priority: 10,
            enabled: true,
            healthy: true,
            consecutive_failures: 0,
            consecutive_successes: 0,
            failure_threshold: 3,
            recovery_threshold: 2,
            bytes_sent: 0,
            packets_sent: 0,
        }
    }
}

impl NetworkInterface {
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_thresholds(mut self, fail: u32, recover: u32) -> Self {
        self.failure_threshold = fail.max(1);
        self.recovery_threshold = recover.max(1);
        self
    }
}

impl NetworkInterface {
    pub fn record_success(&mut self, bytes: usize) {
        self.bytes_sent = self.bytes_sent.saturating_add(bytes as u64);
        self.packets_sent = self.packets_sent.saturating_add(1);
        self.consecutive_failures = 0;
        self.consecutive_successes = self.consecutive_successes.saturating_add(1);
        if !self.healthy && self.consecutive_successes >= self.recovery_threshold {
            self.healthy = true;
        }
    }

    pub fn record_failure(&mut self) -> bool {
        self.consecutive_successes = 0;
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.healthy && self.consecutive_failures >= self.failure_threshold {
            self.healthy = false;
            return true;
        }
        false
    }
}

/// Configuration for failover routing engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterConfig {
    pub policy: FailoverPolicy,
    pub max_payload_size: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            policy: FailoverPolicy::BroadcastWithUnicastBackup,
            max_payload_size: 65507,
        }
    }
}

/// Routing decision output describing transport route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub interface_id: String,
    pub kind: RouteKind,
    pub destination: SocketAddr,
    pub payload_len: usize,
    pub is_failover: bool,
}

/// Multi-interface broadcast and unicast failover router.
#[derive(Debug, Clone)]
pub struct FailoverRouter {
    config: RouterConfig,
    interfaces: Vec<NetworkInterface>,
    round_robin_idx: usize,
    stats: RouterStats,
}

impl FailoverRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            config,
            interfaces: Vec::new(),
            round_robin_idx: 0,
            stats: RouterStats::default(),
        }
    }

    pub fn with_interfaces(mut self, interfaces: Vec<NetworkInterface>) -> Self {
        self.interfaces = interfaces;
        self
    }
}

impl FailoverRouter {
    pub fn add_interface(&mut self, iface: NetworkInterface) {
        self.interfaces.retain(|i| i.id != iface.id);
        self.interfaces.push(iface);
    }

    pub fn remove_interface(&mut self, id: &str) -> bool {
        let prev = self.interfaces.len();
        self.interfaces.retain(|i| i.id != id);
        self.interfaces.len() < prev
    }

    pub fn get_interface(&self, id: &str) -> Option<&NetworkInterface> {
        self.interfaces.iter().find(|i| i.id == id)
    }

    pub fn get_interface_mut(&mut self, id: &str) -> Option<&mut NetworkInterface> {
        self.interfaces.iter_mut().find(|i| i.id == id)
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(iface) = self.get_interface_mut(id) {
            iface.enabled = enabled;
            return true;
        }
        false
    }
}

impl FailoverRouter {
    fn select_index(&mut self) -> Result<(usize, bool), RouterError> {
        let mut h: Vec<usize> = self
            .interfaces
            .iter()
            .enumerate()
            .filter(|(_, i)| i.enabled && i.healthy)
            .map(|(k, _)| k)
            .collect();
        let is_probe = h.is_empty();
        if is_probe {
            h = self
                .interfaces
                .iter()
                .enumerate()
                .filter(|(_, i)| i.enabled)
                .map(|(k, _)| k)
                .collect();
        }
        if h.is_empty() {
            return Err(RouterError::NoHealthyInterface);
        }
        match self.config.policy {
            FailoverPolicy::PriorityPrimary => {
                let best = *h
                    .iter()
                    .min_by_key(|&&i| self.interfaces[i].priority)
                    .unwrap();
                let min_pri = self
                    .interfaces
                    .iter()
                    .filter(|i| i.enabled)
                    .map(|i| i.priority)
                    .min()
                    .unwrap_or(0);
                Ok((best, self.interfaces[best].priority > min_pri))
            }
            FailoverPolicy::RoundRobin => {
                let idx = h[self.round_robin_idx % h.len()];
                self.round_robin_idx = self.round_robin_idx.wrapping_add(1);
                Ok((idx, false))
            }
            FailoverPolicy::BroadcastWithUnicastBackup => {
                if let Some(&b) = h
                    .iter()
                    .find(|&&i| self.interfaces[i].kind == RouteKind::Broadcast)
                {
                    Ok((b, false))
                } else {
                    Ok((h[0], true))
                }
            }
        }
    }
}

impl FailoverRouter {
    fn validate_payload(&mut self, payload: &[u8]) -> Result<(), RouterError> {
        self.stats.packets_received = self.stats.packets_received.saturating_add(1);
        if payload.is_empty() {
            self.stats.packets_dropped = self.stats.packets_dropped.saturating_add(1);
            return Err(RouterError::EmptyPayload);
        }
        if payload.len() > self.config.max_payload_size {
            self.stats.packets_dropped = self.stats.packets_dropped.saturating_add(1);
            return Err(RouterError::PayloadTooLarge {
                size: payload.len(),
                limit: self.config.max_payload_size,
            });
        }
        Ok(())
    }
}

impl FailoverRouter {
    pub fn route(&mut self, payload: &[u8]) -> Result<RoutingDecision, RouterError> {
        self.validate_payload(payload)?;
        let (idx, is_fo) = self.select_index().inspect_err(|_| {
            self.stats.packets_dropped = self.stats.packets_dropped.saturating_add(1);
        })?;
        let iface = &self.interfaces[idx];
        if is_fo {
            self.stats.failover_events = self.stats.failover_events.saturating_add(1);
        }
        match iface.kind {
            RouteKind::Broadcast => self.stats.broadcast_dispatched += 1,
            RouteKind::Unicast => self.stats.unicast_dispatched += 1,
        }
        self.stats.packets_routed += 1;
        self.stats.total_bytes_routed += payload.len() as u64;
        Ok(RoutingDecision {
            interface_id: iface.id.clone(),
            kind: iface.kind,
            destination: iface.destination,
            payload_len: payload.len(),
            is_failover: is_fo,
        })
    }
}

impl FailoverRouter {
    pub fn record_success(&mut self, id: &str, bytes: usize) -> bool {
        if let Some(iface) = self.get_interface_mut(id) {
            iface.record_success(bytes);
            return true;
        }
        false
    }

    pub fn record_failure(&mut self, id: &str) -> bool {
        if let Some(iface) = self.get_interface_mut(id) {
            return iface.record_failure();
        }
        false
    }
}

impl FailoverRouter {
    pub fn stats(&self) -> &RouterStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = RouterStats::default();
    }

    pub fn healthy_count(&self) -> usize {
        self.interfaces
            .iter()
            .filter(|i| i.enabled && i.healthy)
            .count()
    }

    pub fn interfaces(&self) -> &[NetworkInterface] {
        &self.interfaces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_health_transitions() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut iface = NetworkInterface::new("eth0", "primary", RouteKind::Broadcast, addr)
            .with_thresholds(2, 2);

        assert!(iface.healthy);
        assert!(!iface.record_failure());
        assert!(iface.healthy);
        assert!(iface.record_failure());
        assert!(!iface.healthy);

        iface.record_success(100);
        assert!(!iface.healthy);
        iface.record_success(100);
        assert!(iface.healthy);
        assert_eq!(iface.bytes_sent, 200);
        assert_eq!(iface.packets_sent, 2);
    }

    #[test]
    fn test_priority_primary_selection() {
        let mut r = FailoverRouter::new(RouterConfig {
            policy: FailoverPolicy::PriorityPrimary,
            max_payload_size: 1024,
        });
        let a1 = "127.0.0.1:8001".parse().unwrap();
        let a2 = "127.0.0.1:8002".parse().unwrap();
        r.add_interface(
            NetworkInterface::new("p1", "main", RouteKind::Broadcast, a1).with_priority(5),
        );
        r.add_interface(
            NetworkInterface::new("p2", "bak", RouteKind::Unicast, a2).with_priority(20),
        );

        let dec = r.route(b"cot1").unwrap();
        assert_eq!(dec.interface_id, "p1");
        assert!(!dec.is_failover);

        r.get_interface_mut("p1").unwrap().healthy = false;
        let dec2 = r.route(b"cot2").unwrap();
        assert_eq!(dec2.interface_id, "p2");
        assert!(dec2.is_failover);
    }

    #[test]
    fn test_broadcast_with_unicast_failover() {
        let mut r = FailoverRouter::new(RouterConfig::default());
        let b_addr = "239.2.3.1:6969".parse().unwrap();
        let u_addr = "10.0.0.5:8087".parse().unwrap();
        r.add_interface(NetworkInterface::new(
            "bcast",
            "bcast",
            RouteKind::Broadcast,
            b_addr,
        ));
        r.add_interface(NetworkInterface::new(
            "ucast",
            "ucast",
            RouteKind::Unicast,
            u_addr,
        ));

        let d1 = r.route(b"cot_broadcast").unwrap();
        assert_eq!(d1.interface_id, "bcast");
        assert_eq!(d1.kind, RouteKind::Broadcast);
        assert!(!d1.is_failover);

        r.get_interface_mut("bcast").unwrap().healthy = false;
        let d2 = r.route(b"cot_unicast").unwrap();
        assert_eq!(d2.interface_id, "ucast");
        assert_eq!(d2.kind, RouteKind::Unicast);
        assert!(d2.is_failover);
    }

    #[test]
    fn test_round_robin_distribution() {
        let mut r = FailoverRouter::new(RouterConfig {
            policy: FailoverPolicy::RoundRobin,
            max_payload_size: 1024,
        });
        let a1 = "127.0.0.1:8001".parse().unwrap();
        let a2 = "127.0.0.1:8002".parse().unwrap();
        r.add_interface(NetworkInterface::new("i1", "if1", RouteKind::Broadcast, a1));
        r.add_interface(NetworkInterface::new("i2", "if2", RouteKind::Unicast, a2));

        let d1 = r.route(b"p1").unwrap();
        let d2 = r.route(b"p2").unwrap();
        let d3 = r.route(b"p3").unwrap();
        assert_eq!(d1.interface_id, "i1");
        assert_eq!(d2.interface_id, "i2");
        assert_eq!(d3.interface_id, "i1");
    }

    #[test]
    fn test_payload_validation_and_exhaustion_errors() {
        let mut r = FailoverRouter::new(RouterConfig {
            policy: FailoverPolicy::PriorityPrimary,
            max_payload_size: 10,
        });
        assert_eq!(r.route(b""), Err(RouterError::EmptyPayload));
        assert_eq!(
            r.route(b"12345678901"),
            Err(RouterError::PayloadTooLarge {
                size: 11,
                limit: 10
            })
        );
        assert_eq!(r.route(b"valid"), Err(RouterError::NoHealthyInterface));
    }

    #[test]
    fn test_router_stats_tracking_and_reset() {
        let mut r = FailoverRouter::new(RouterConfig::default());
        let a1 = "127.0.0.1:8001".parse().unwrap();
        let a2 = "127.0.0.1:8002".parse().unwrap();
        r.add_interface(NetworkInterface::new(
            "b1",
            "bcast",
            RouteKind::Broadcast,
            a1,
        ));
        r.add_interface(NetworkInterface::new("u1", "ucast", RouteKind::Unicast, a2));

        r.route(b"hello").unwrap();
        r.record_success("b1", 5);
        r.get_interface_mut("b1").unwrap().healthy = false;
        r.route(b"world!").unwrap();

        assert_eq!(r.stats().packets_received, 2);
        assert_eq!(r.stats().packets_routed, 2);
        assert_eq!(r.stats().broadcast_dispatched, 1);
        assert_eq!(r.stats().unicast_dispatched, 1);
        assert_eq!(r.stats().failover_events, 1);
        assert_eq!(r.stats().total_bytes_routed, 11);

        r.reset_stats();
        assert_eq!(r.stats().packets_routed, 0);
        assert_eq!(r.stats().total_bytes_routed, 0);
    }

    #[test]
    fn test_interface_mutation_and_counts() {
        let mut r = FailoverRouter::new(RouterConfig::default());
        let a = "127.0.0.1:8001".parse().unwrap();
        r.add_interface(NetworkInterface::new(
            "eth0",
            "lan",
            RouteKind::Broadcast,
            a,
        ));
        assert_eq!(r.healthy_count(), 1);
        assert_eq!(r.interfaces().len(), 1);

        assert!(r.set_enabled("eth0", false));
        assert_eq!(r.healthy_count(), 0);
        assert!(!r.set_enabled("nonexistent", true));

        assert!(r.remove_interface("eth0"));
        assert_eq!(r.interfaces().len(), 0);
        assert!(!r.remove_interface("eth0"));
    }
}
