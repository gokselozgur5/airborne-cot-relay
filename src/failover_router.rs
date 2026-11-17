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
        let h: Vec<usize> = self.interfaces.iter().enumerate()
            .filter(|(_, i)| i.enabled && i.healthy).map(|(k, _)| k).collect();
        if h.is_empty() { return Err(RouterError::NoHealthyInterface); }
        match self.config.policy {
            FailoverPolicy::PriorityPrimary => {
                let best = *h.iter().min_by_key(|&&i| self.interfaces[i].priority).unwrap();
                let min_pri = self.interfaces.iter().map(|i| i.priority).min().unwrap_or(0);
                Ok((best, self.interfaces[best].priority > min_pri))
            }
            FailoverPolicy::RoundRobin => {
                let idx = h[self.round_robin_idx % h.len()];
                self.round_robin_idx = self.round_robin_idx.wrapping_add(1);
                Ok((idx, false))
            }
            FailoverPolicy::BroadcastWithUnicastBackup => {
                if let Some(&b) = h.iter().find(|&&i| self.interfaces[i].kind == RouteKind::Broadcast) {
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
