use crate::config::RelayError;
use crate::relay::CotSink;
use std::net::{SocketAddr, UdpSocket};

/// UDP broadcast sink delivering payloads to a broadcast address.
#[derive(Debug)]
pub struct UdpBroadcastSink {
    socket: UdpSocket,
    target_addr: SocketAddr,
}
