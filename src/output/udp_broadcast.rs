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

/// UDP broadcast sink delivering payloads to a broadcast address.
#[derive(Debug)]
pub struct UdpBroadcastSink {
    socket: UdpSocket,
    target_addr: SocketAddr,
    broadcast: bool,
}

impl UdpBroadcastSink {
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

impl UdpBroadcastSink {
    pub fn bind(target: SocketAddr, cfg: &UdpBroadcastConfig) -> Result<Self, RelayError> {
        let socket = Self::bind_socket(target, cfg.outbound_interface)?;
        socket.set_broadcast(true)?;
        let broadcast = socket.broadcast().unwrap_or(true);
        Ok(Self {
            socket,
            target_addr: target,
            broadcast,
        })
    }

    pub fn with_defaults(target: SocketAddr) -> Result<Self, RelayError> {
        Self::bind(target, &UdpBroadcastConfig::default())
    }
}
