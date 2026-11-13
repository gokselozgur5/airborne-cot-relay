use crate::config::RelayError;
use crate::relay::CotSink;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

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

impl UdpMulticastSink {
    pub fn bind(target: SocketAddr, cfg: &UdpMulticastConfig) -> Result<Self, RelayError> {
        let socket = Self::bind_socket(target, cfg.outbound_interface)?;
        if let Some(ttl) = cfg.ttl {
            socket.set_multicast_ttl_v4(ttl)?;
        }
        if let Some(loopback) = cfg.loopback {
            socket.set_multicast_loop_v4(loopback)?;
        }
        let ttl = socket.multicast_ttl_v4().unwrap_or(1);
        let loopback = socket.multicast_loop_v4().unwrap_or(false);
        Ok(Self {
            socket,
            target_addr: target,
            ttl,
            loopback,
        })
    }

    pub fn with_defaults(target: SocketAddr) -> Result<Self, RelayError> {
        Self::bind(target, &UdpMulticastConfig::default())
    }
}

impl UdpMulticastSink {
    pub fn target_addr(&self) -> SocketAddr {
        self.target_addr
    }

    pub fn ttl(&self) -> u32 {
        self.ttl
    }

    pub fn loopback(&self) -> bool {
        self.loopback
    }

    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}

impl CotSink for UdpMulticastSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.socket
            .send_to(payload, self.target_addr)
            .map_err(RelayError::from)
    }
}
