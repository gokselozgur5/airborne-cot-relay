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
