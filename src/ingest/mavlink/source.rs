//! UDP socket source for MAVLink v2 telemetry.

use super::frame::decode_mavlink_v2_frame;
use super::synth::MavlinkPoseSynthesizer;
use crate::config::RelayError;
use crate::runtime::source::TelemetrySource;
use crate::telemetry::PlatformPose;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct MavlinkUdpSource {
    socket: UdpSocket,
    buffer: Vec<u8>,
    synthesizer: MavlinkPoseSynthesizer,
}

impl MavlinkUdpSource {
    pub fn new(socket: UdpSocket) -> Result<Self, RelayError> {
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            buffer: vec![0u8; 65535],
            synthesizer: MavlinkPoseSynthesizer::default(),
        })
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, RelayError> {
        let socket = UdpSocket::bind(addr)?;
        Self::new(socket)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, RelayError> {
        Ok(self.socket.local_addr()?)
    }

    pub fn poll_pose(&mut self) -> Result<Option<PlatformPose>, RelayError> {
        loop {
            match self.socket.recv_from(&mut self.buffer) {
                Ok((len, _sender)) => {
                    if let Ok(pkt) = decode_mavlink_v2_frame(&self.buffer[..len]) {
                        if let Some(pose) = self.synthesizer.update(&pkt) {
                            return Ok(Some(pose));
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(RelayError::IoError(e.to_string())),
            }
        }
    }
}

impl TelemetrySource for MavlinkUdpSource {
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        match self.socket.recv_from(&mut self.buffer) {
            Ok((len, _sender)) => Ok(Some(self.buffer[..len].to_vec())),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(RelayError::IoError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::mavlink::frame::*;

    #[test]
    fn test_udp_loopback_pose() {
        let mut rx = MavlinkUdpSource::bind("127.0.0.1:0").expect("bind rx");
        let rx_addr = rx.local_addr().unwrap();
        let tx = UdpSocket::bind("127.0.0.1:0").expect("bind tx");

        let att = encode_mavlink_v2_attitude(1, 1, 1, 500, 0.0, 0.0, 0.5);
        let pos = encode_mavlink_v2_global_position(
            2, 1, 1, 520, 399000000, 328000000, 100000, 50000, 0, 0, 0, 0,
        );

        tx.send_to(&att, rx_addr).unwrap();
        tx.send_to(&pos, rx_addr).unwrap();

        let mut received_pose = None;
        for _ in 0..10 {
            if let Ok(Some(pose)) = rx.poll_pose() {
                received_pose = Some(pose);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let p = received_pose.expect("received pose over loopback");
        assert!((p.lat_deg - 39.9).abs() < 1e-6);
        assert!((p.lon_deg - 32.8).abs() < 1e-6);
    }
}
