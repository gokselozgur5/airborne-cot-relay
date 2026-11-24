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
        let mut budget = 32;
        while budget > 0 {
            budget -= 1;
            match self.socket.recv_from(&mut self.buffer) {
                Ok((len, _sender)) => {
                    if let Ok(pkt) = decode_mavlink_v2_frame(&self.buffer[..len]) {
                        if let Some(pose) = self.synthesizer.update(&pkt) {
                            return Ok(Some(pose));
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                {
                    return Ok(None)
                }
                Err(e) => return Err(RelayError::IoError(e.to_string())),
            }
        }
        Ok(None)
    }
}

impl TelemetrySource for MavlinkUdpSource {
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        if let Some(pose) = self.poll_pose()? {
            let telem = stm32_navcore::protocol::NavTelemetry {
                timestamp_ms: (pose.timestamp_ms & 0xFFFF_FFFF) as u32,
                pos_ned_m: [0.0, 0.0, -pose.hae_m as f32],
                vel_ned_mps: [0.0; 3],
                euler_rad: [
                    pose.roll_rad as f32,
                    pose.pitch_rad as f32,
                    pose.yaw_rad as f32,
                ],
                status: 1,
            };
            let mut buf = [0u8; stm32_navcore::protocol::NavTelemetry::MAX_UART_FRAME_LEN];
            let len = telem
                .pack_uart(&mut buf)
                .map_err(|e| RelayError::IoError(format!("pack_uart: {e:?}")))?;
            return Ok(Some(buf[..len].to_vec()));
        }
        Ok(None)
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

    #[test]
    fn test_mavlink_udp_source_as_telemetry_source() {
        let mut rx = MavlinkUdpSource::bind("127.0.0.1:0").expect("bind rx");
        let rx_addr = rx.local_addr().unwrap();
        let tx = UdpSocket::bind("127.0.0.1:0").expect("bind tx");

        let att = encode_mavlink_v2_attitude(1, 1, 1, 500, 0.0, 0.0, 0.5);
        let pos = encode_mavlink_v2_global_position(
            2, 1, 1, 520, 399000000, 328000000, 100000, 50000, 0, 0, 0, 0,
        );

        tx.send_to(&att, rx_addr).unwrap();
        tx.send_to(&pos, rx_addr).unwrap();

        let mut chunk = None;
        for _ in 0..10 {
            if let Ok(Some(bytes)) = rx.poll_chunk() {
                chunk = Some(bytes);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let raw = chunk.expect("poll_chunk returns packed uart bytes");
        let telem = stm32_navcore::protocol::NavTelemetry::unpack_uart(&raw).expect("unpack uart");
        assert_eq!(telem.timestamp_ms, 520);
        assert_eq!(telem.pos_ned_m[2], -100.0);
    }
}
