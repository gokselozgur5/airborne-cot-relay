//! Non-blocking TCP stream source for telemetry bytes.

use crate::config::RelayError;
use crate::runtime::source::TelemetrySource;
use std::io::{ErrorKind, Read};
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

    pub fn local_addr(&self) -> Result<SocketAddr, RelayError> {
        Ok(self.stream.local_addr()?)
    }

    pub fn poll(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        self.poll_chunk()
    }
}

impl TelemetrySource for TcpTelemetrySource {
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        match self.stream.read(&mut self.buffer) {
            Ok(0) => Err(RelayError::IoError("connection closed by peer".into())),
            Ok(n) => Ok(Some(self.buffer[..n].to_vec())),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                Ok(None)
            }
            Err(e) => Err(RelayError::from(e)),
        }
    }
}
