//! Canonical multicast receiver adapter delegating to MulticastListener.

use std::net::{SocketAddr, UdpSocket};

pub use crate::multicast_listener::{
    MulticastListener, MulticastListenerConfig as MulticastRxConfig,
    MulticastListenerError as MulticastRxError,
};

#[derive(Debug)]
pub struct MulticastReceiver {
    inner: MulticastListener,
}

impl MulticastReceiver {
    pub fn bind(config: MulticastRxConfig) -> Result<Self, MulticastRxError> {
        let inner = MulticastListener::bind(config)?;
        Ok(Self { inner })
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), MulticastRxError> {
        self.inner.socket().recv_from(buf).map_err(MulticastRxError::Io)
    }

    pub fn recv_datagram(&self) -> Result<(Vec<u8>, SocketAddr), MulticastRxError> {
        let max_size = self.inner.config().max_packet_size;
        let mut buf = vec![0u8; max_size.saturating_add(1)];
        let (bytes_read, src) = self.recv(&mut buf)?;
        if bytes_read > max_size {
            return Err(MulticastRxError::OversizedDatagram(bytes_read));
        }
        buf.truncate(bytes_read);
        Ok((buf, src))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, MulticastRxError> {
        self.inner.socket().local_addr().map_err(MulticastRxError::Io)
    }

    pub fn socket(&self) -> &UdpSocket {
        self.inner.socket()
    }

    pub fn config(&self) -> &MulticastRxConfig {
        self.inner.config()
    }

    pub fn is_joined(&self) -> bool {
        true
    }

    pub fn leave_group(&mut self) -> Result<(), MulticastRxError> {
        let cfg = self.inner.config();
        self.inner.socket().leave_multicast_v4(&cfg.multicast_addr, &cfg.interface_addr)?;
        Ok(())
    }

    #[test]
    fn test_config_validation_invalid_group() {
        let unicast: Ipv4Addr = "192.168.1.1".parse().unwrap();
        let cfg = MulticastRxConfig::new(unicast, 8080);
        assert!(matches!(
            cfg.validate(),
            Err(MulticastRxError::InvalidGroup(_))
        ));
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
        assert!(matches!(
            cfg.validate(),
            Err(MulticastRxError::InvalidBufferSize)
        ));
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

    #[test]
    fn test_multicast_rx_packet_reception() {
        let group: Ipv4Addr = "239.255.0.3".parse().unwrap();
        let temp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let port = temp.local_addr().unwrap().port();
        drop(temp);

        let cfg = MulticastRxConfig::new(group, port).with_read_timeout(Duration::from_millis(200));
        let rx = MulticastReceiver::bind(cfg).unwrap();

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        sender
            .send_to(
                b"COT_MULTICAST_TEST",
                SocketAddr::from(([127, 0, 0, 1], port)),
            )
            .unwrap();

        let (payload, _src) = rx.recv_datagram().unwrap();
        assert_eq!(&payload, b"COT_MULTICAST_TEST");
    }
}
