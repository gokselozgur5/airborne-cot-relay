use std::fmt;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

#[derive(Debug)]
pub enum MulticastRxError {
    InvalidGroup(Ipv4Addr),
    InvalidPort,
    InvalidBufferSize,
    Io(io::Error),
}

impl fmt::Display for MulticastRxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGroup(ip) => write!(f, "invalid multicast address: {}", ip),
            Self::InvalidPort => write!(f, "multicast port must be non-zero"),
            Self::InvalidBufferSize => write!(f, "buffer size must be non-zero"),
            Self::Io(err) => write!(f, "multicast io error: {}", err),
        }
    }
}

impl std::error::Error for MulticastRxError {}

impl From<io::Error> for MulticastRxError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
