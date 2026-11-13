//! Non-blocking UDP socket source for telemetry datagrams.

use crate::config::RelayError;
use crate::runtime::source::TelemetrySource;
use std::io::ErrorKind;
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

    pub fn poll(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        self.poll_chunk()
    }
}

impl TelemetrySource for UdpTelemetrySource {
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        match self.socket.recv_from(&mut self.buffer) {
            Ok((n, _src)) => Ok(Some(self.buffer[..n].to_vec())),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                Ok(None)
            }
            Err(e) => Err(RelayError::from(e)),
        }
    }
}
