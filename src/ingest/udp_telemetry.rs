//! Non-blocking UDP socket source for telemetry datagrams.

use crate::config::RelayError;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct UdpTelemetrySource {
    socket: UdpSocket,
    buffer: Vec<u8>,
}

impl UdpTelemetrySource {
    pub fn new(socket: UdpSocket) -> Result<Self, RelayError> {
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            buffer: vec![0u8; 65535],
        })
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, RelayError> {
        let socket = UdpSocket::bind(addr)?;
        Self::new(socket)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, RelayError> {
        Ok(self.socket.local_addr()?)
    }
}
