use crate::config::RelayError;
use crate::relay::CotSink;
use std::net::{SocketAddr, UdpSocket};

/// UDP broadcast sink delivering payloads to a broadcast address.
#[derive(Debug)]
pub struct UdpBroadcastSink {
    socket: UdpSocket,
    target_addr: SocketAddr,
}

impl UdpBroadcastSink {
    pub fn bind(target: SocketAddr) -> Result<Self, RelayError> {
        let bind_addr = match target {
            SocketAddr::V4(_) => SocketAddr::from(([0, 0, 0, 0], 0)),
            SocketAddr::V6(_) => SocketAddr::from(([0; 16], 0)),
        };
        let socket = UdpSocket::bind(bind_addr).map_err(RelayError::from)?;
        socket.set_broadcast(true).map_err(RelayError::from)?;
        Ok(Self {
            socket,
            target_addr: target,
        })
    }
}

impl UdpBroadcastSink {
    pub fn target_addr(&self) -> SocketAddr {
        self.target_addr
    }

    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}

impl CotSink for UdpBroadcastSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.socket
            .send_to(payload, self.target_addr)
            .map_err(RelayError::from)
    }
}
