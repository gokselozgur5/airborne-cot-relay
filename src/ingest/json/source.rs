//! UDP detection source streaming JSON-lines.

use super::parser::parse_json_detection;
use crate::config::RelayError;
use crate::runtime::source::DetectionSource;
use crate::targeter::TargetDetection;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct JsonUdpDetectionSource {
    socket: UdpSocket,
    buffer: Vec<u8>,
    bad_lines: u64,
}

impl JsonUdpDetectionSource {
    pub fn new(socket: UdpSocket) -> Result<Self, RelayError> {
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            buffer: vec![0u8; 65535],
            bad_lines: 0,
        })
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, RelayError> {
        let socket = UdpSocket::bind(addr)?;
        Self::new(socket)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, RelayError> {
        Ok(self.socket.local_addr()?)
    }

    pub fn bad_lines(&self) -> u64 {
        self.bad_lines
    }
}

impl DetectionSource for JsonUdpDetectionSource {
    fn poll_batch(&mut self) -> Result<Option<Vec<TargetDetection>>, RelayError> {
        match self.socket.recv_from(&mut self.buffer) {
            Ok((len, _sender)) => {
                let text = match std::str::from_utf8(&self.buffer[..len]) {
                    Ok(s) => s,
                    Err(_) => {
                        self.bad_lines += 1;
                        return Ok(None);
                    }
                };
                let mut batch = Vec::new();
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match parse_json_detection(trimmed) {
                        Ok(det) => batch.push(det),
                        Err(_) => self.bad_lines += 1,
                    }
                }
                if batch.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(batch))
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(RelayError::IoError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_udp_detection_source_loopback() {
        let mut rx = JsonUdpDetectionSource::bind("127.0.0.1:0").expect("bind rx");
        let rx_addr = rx.local_addr().unwrap();
        let tx = UdpSocket::bind("127.0.0.1:0").expect("bind tx");

        let payload = b"{\"u\":10.0,\"v\":20.0,\"x1\":5.0,\"y1\":15.0,\"x2\":15.0,\"y2\":25.0,\"class\":\"truck\",\"conf\":0.85}\n{\"u\":50.0,\"v\":60.0,\"x1\":40.0,\"y1\":50.0,\"x2\":60.0,\"y2\":70.0,\"class\":\"car\",\"conf\":0.9}\n";
        tx.send_to(payload, rx_addr).unwrap();

        let mut batch = None;
        for _ in 0..10 {
            if let Ok(Some(b)) = rx.poll_batch() {
                batch = Some(b);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let b = batch.expect("received batch");
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].classification, "truck");
        assert_eq!(b[1].classification, "car");
    }
}
