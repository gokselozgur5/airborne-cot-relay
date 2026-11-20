//! Non-blocking UDP socket source for telemetry datagrams.

use crate::config::RelayError;
use crate::ingest::SourceFilter;
use crate::runtime::source::TelemetrySource;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct UdpTelemetrySource {
    socket: UdpSocket,
    buffer: Vec<u8>,
    filter: Option<SourceFilter>,
}

impl UdpTelemetrySource {
    pub fn new(socket: UdpSocket) -> Result<Self, RelayError> {
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            buffer: vec![0u8; 65535],
            filter: None,
        })
    }

    pub fn with_source_filter(mut self, filter: SourceFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn rejected_count(&self) -> u64 {
        self.filter.as_ref().map_or(0, |f| f.rejected_count())
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
        loop {
            match self.socket.recv_from(&mut self.buffer) {
                Ok((n, src)) => {
                    if let Some(ref mut filter) = self.filter {
                        if !filter.check(&src.ip()) {
                            continue;
                        }
                    }
                    return Ok(Some(self.buffer[..n].to_vec()));
                }
                Err(ref e)
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                {
                    return Ok(None);
                }
                Err(e) => return Err(RelayError::from(e)),
            }
        }
    }
}
