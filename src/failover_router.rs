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
