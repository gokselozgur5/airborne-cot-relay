use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use crate::config::RelayError;
use crate::relay::CotSink;

/// Configuration options for UDP multicast egress.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UdpMulticastConfig {
    pub ttl: Option<u32>,
    pub loopback: Option<bool>,
    pub outbound_interface: Option<Ipv4Addr>,
}

impl UdpMulticastConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_loopback(mut self, loopback: bool) -> Self {
        self.loopback = Some(loopback);
        self
    }

    pub fn with_outbound_interface(mut self, addr: Ipv4Addr) -> Self {
        self.outbound_interface = Some(addr);
        self
    }
}
