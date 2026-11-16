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

impl MulticastReceiver {
    pub fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), MulticastRxError> {
        self.socket.recv_from(buf).map_err(MulticastRxError::Io)
    }

    pub fn recv_datagram(&self) -> Result<(Vec<u8>, SocketAddr), MulticastRxError> {
        let mut buf = vec![0u8; self.config.buffer_size];
        let (bytes_read, src) = self.recv(&mut buf)?;
        buf.truncate(bytes_read);
        Ok((buf, src))
    }
}

impl MulticastReceiver {
    pub fn leave_group(&mut self) -> Result<(), MulticastRxError> {
        if self.joined {
            self.socket
                .leave_multicast_v4(&self.config.multicast_addr, &self.config.interface_addr)?;
            self.joined = false;
        }
        Ok(())
    }

    pub fn rejoin_group(&mut self) -> Result<(), MulticastRxError> {
        if !self.joined {
            self.socket
                .join_multicast_v4(&self.config.multicast_addr, &self.config.interface_addr)?;
            self.joined = true;
        }
        Ok(())
    }
}

impl MulticastReceiver {
    pub fn local_addr(&self) -> Result<SocketAddr, MulticastRxError> {
        self.socket.local_addr().map_err(MulticastRxError::Io)
    }

    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    pub fn config(&self) -> &MulticastRxConfig {
        &self.config
    }

    pub fn is_joined(&self) -> bool {
        self.joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_valid() {
        let group: Ipv4Addr = "239.2.3.1".parse().unwrap();
        let cfg = MulticastRxConfig::new(group, 8080)
            .with_buffer_size(4096)
            .with_read_timeout(Duration::from_millis(100));
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_group() {
        let unicast: Ipv4Addr = "192.168.1.1".parse().unwrap();
        let cfg = MulticastRxConfig::new(unicast, 8080);
        assert!(matches!(cfg.validate(), Err(MulticastRxError::InvalidGroup(_))));
    }

    #[test]
    fn test_config_validation_zero_port() {
        let group: Ipv4Addr = "239.2.3.1".parse().unwrap();
        let cfg = MulticastRxConfig::new(group, 0);
        assert!(matches!(cfg.validate(), Err(MulticastRxError::InvalidPort)));
    }

    #[test]
    fn test_config_validation_zero_buffer() {
        let group: Ipv4Addr = "239.2.3.1".parse().unwrap();
        let cfg = MulticastRxConfig::new(group, 8080).with_buffer_size(0);
        assert!(matches!(cfg.validate(), Err(MulticastRxError::InvalidBufferSize)));
    }

    #[test]
    fn test_multicast_rx_bind_and_accessors() {
        let group: Ipv4Addr = "239.255.0.1".parse().unwrap();
        let temp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = temp.local_addr().unwrap().port();
        drop(temp);

        let cfg = MulticastRxConfig::new(group, port).with_buffer_size(1024);
        let rx = MulticastReceiver::bind(cfg.clone()).unwrap();
        assert_eq!(rx.config().multicast_addr, group);
        assert_eq!(rx.config().buffer_size, 1024);
        assert!(rx.is_joined());
        assert!(rx.local_addr().is_ok());
    }

    #[test]
    fn test_multicast_rx_leave_and_rejoin() {
        let group: Ipv4Addr = "239.255.0.2".parse().unwrap();
        let temp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = temp.local_addr().unwrap().port();
        drop(temp);

        let cfg = MulticastRxConfig::new(group, port);
        let mut rx = MulticastReceiver::bind(cfg).unwrap();
        assert!(rx.is_joined());
        assert!(rx.leave_group().is_ok());
        assert!(!rx.is_joined());
        assert!(rx.rejoin_group().is_ok());
        assert!(rx.is_joined());
    }
