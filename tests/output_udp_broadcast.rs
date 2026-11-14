use airborne_cot_relay::output::{CotSink, CotSource};
use std::net::{SocketAddr, UdpSocket};

pub struct UdpBroadcastSink {
    socket: UdpSocket,
}

impl UdpBroadcastSink {
    pub fn new() -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:12345")?;
        socket.set_broadcast(true)?;
        Ok(Self socket)
    }

    pub async fn send(&self, payload: &[u8]) -> Result<(), std::io::Error> {
        self.socket.send_to(payload, "255.255.255.255".parse().unwrap())?;
        Ok(())
    }
}

impl CotSink for UdpBroadcastSink {
    fn send(&self, payload: &[u8]) -> Result<(), std::io::Error> {
        self.send(payload)
    }
