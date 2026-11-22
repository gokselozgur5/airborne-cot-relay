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

    pub fn rejoin_group(&mut self) -> Result<(), MulticastRxError> {
        let cfg = self.inner.config();
        self.inner.socket().join_multicast_v4(&cfg.multicast_addr, &cfg.interface_addr)?;
        Ok(())
    }
}
