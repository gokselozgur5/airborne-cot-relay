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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticastRxConfig {
    pub multicast_addr: Ipv4Addr,
    pub interface_addr: Ipv4Addr,
    pub port: u16,
    pub read_timeout: Option<Duration>,
    pub buffer_size: usize,
}

impl MulticastRxConfig {
    pub fn new(multicast_addr: Ipv4Addr, port: u16) -> Self {
        Self {
            multicast_addr,
            interface_addr: Ipv4Addr::UNSPECIFIED,
            port,
            read_timeout: None,
            buffer_size: 2048,
        }
    }

    pub fn with_interface(mut self, interface_addr: Ipv4Addr) -> Self {
        self.interface_addr = interface_addr;
        self
    }

    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

impl MulticastRxConfig {
    pub fn validate(&self) -> Result<(), MulticastRxError> {
        if !self.multicast_addr.is_multicast() {
            return Err(MulticastRxError::InvalidGroup(self.multicast_addr));
        }
        if self.port == 0 {
            return Err(MulticastRxError::InvalidPort);
        }
        if self.buffer_size == 0 {
            return Err(MulticastRxError::InvalidBufferSize);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct MulticastReceiver {
    socket: UdpSocket,
    config: MulticastRxConfig,
    joined: bool,
}

impl MulticastReceiver {
    fn bind_socket(config: &MulticastRxConfig) -> io::Result<UdpSocket> {
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
        let socket = UdpSocket::bind(bind_addr)?;
        if let Some(timeout) = config.read_timeout {
            socket.set_read_timeout(Some(timeout))?;
        }
        Ok(socket)
    }
}

impl MulticastReceiver {
    pub fn bind(config: MulticastRxConfig) -> Result<Self, MulticastRxError> {
        config.validate()?;
        let socket = Self::bind_socket(&config)?;
        socket.join_multicast_v4(&config.multicast_addr, &config.interface_addr)?;
        Ok(Self {
            socket,
            config,
            joined: true,
        })
    }
}
