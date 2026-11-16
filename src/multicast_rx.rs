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
