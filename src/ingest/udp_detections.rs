//! Non-blocking UDP socket source for target detections.

use crate::config::RelayError;
use crate::runtime::source::DetectionSource;
use crate::targeter::TargetDetection;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

fn parse_f64(s: &str, field: &str) -> Result<f64, RelayError> {
    s.parse()
        .map_err(|e| RelayError::DecodeError(format!("{field}: {e}")))
}

fn parse_f32(s: &str, field: &str) -> Result<f32, RelayError> {
    s.parse()
        .map_err(|e| RelayError::DecodeError(format!("{field}: {e}")))
}

pub fn parse_detection_line(line: &str) -> Result<TargetDetection, RelayError> {
    let p: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    let (u, v, x, y, w, h, c, q) = match p.len() {
        8 => (p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]),
        9 => (p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8]),
        _ => return Err(RelayError::DecodeError("invalid field count".into())),
    };
    Ok(TargetDetection {
        pixel_u: parse_f64(u, "pixel_u")?,
        pixel_v: parse_f64(v, "pixel_v")?,
        bbox: (
            parse_f64(x, "x")?,
            parse_f64(y, "y")?,
            parse_f64(w, "w")?,
            parse_f64(h, "h")?,
        ),
        classification: c.to_string(),
        confidence: parse_f32(q, "confidence")?,
        embedding: None,
    })
}

pub struct UdpDetectionSource {
    socket: UdpSocket,
    buffer: Vec<u8>,
    bad_lines: u64,
}

impl UdpDetectionSource {
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

    pub fn poll(&mut self) -> Result<Option<Vec<TargetDetection>>, RelayError> {
        self.poll_batch()
    }
}

pub fn parse_detection_lines(text: &str) -> (Vec<TargetDetection>, u64) {
    let mut batch = Vec::new();
    let mut bad = 0u64;
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() {
            match parse_detection_line(t) {
                Ok(det) => batch.push(det),
                Err(_) => bad += 1,
            }
        }
    }
    (batch, bad)
}

impl DetectionSource for UdpDetectionSource {
    fn poll_batch(&mut self) -> Result<Option<Vec<TargetDetection>>, RelayError> {
        let n = match self.socket.recv_from(&mut self.buffer) {
            Ok((n, _)) => n,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                return Ok(None);
            }
            Err(e) => return Err(RelayError::from(e)),
        };
        let text = match std::str::from_utf8(&self.buffer[..n]) {
            Ok(s) => s,
            Err(_) => {
                self.bad_lines += 1;
                return Ok(Some(Vec::new()));
            }
        };
        let (batch, bad) = parse_detection_lines(text);
        self.bad_lines += bad;
        Ok(Some(batch))
    }
}
