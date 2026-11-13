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

/// UDP multicast sink delivering payloads to a target address.
#[derive(Debug)]
pub struct UdpMulticastSink {
    socket: UdpSocket,
    target_addr: SocketAddr,
    ttl: u32,
    loopback: bool,
}

impl UdpMulticastSink {
    fn bind_socket(target: SocketAddr, iface: Option<Ipv4Addr>) -> Result<UdpSocket, RelayError> {
        let bind_addr = match iface {
            Some(ip) => SocketAddr::from((ip, 0)),
            None => match target {
                SocketAddr::V4(_) => SocketAddr::from(([0, 0, 0, 0], 0)),
                SocketAddr::V6(_) => SocketAddr::from(([0; 16], 0)),
            },
        };
        UdpSocket::bind(bind_addr).map_err(RelayError::from)
    }
}
