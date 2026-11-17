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
