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
