//! Non-blocking TCP stream source for telemetry bytes.

use crate::config::RelayError;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

pub struct TcpTelemetrySource {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl TcpTelemetrySource {
    pub fn new(stream: TcpStream) -> Result<Self, RelayError> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buffer: vec![0u8; 65535],
        })
    }

    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, RelayError> {
        let stream = TcpStream::connect(addr)?;
        Self::new(stream)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, RelayError> {
        Ok(self.stream.peer_addr()?)
    }
}
