use crate::config::RelayError;
use crate::relay::CotSink;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

/// Configuration options for UDP broadcast egress.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UdpBroadcastConfig {
    pub outbound_interface: Option<Ipv4Addr>,
}

impl UdpBroadcastConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_outbound_interface(mut self, addr: Ipv4Addr) -> Self {
        self.outbound_interface = Some(addr);
        self
    }
}
