//! Streaming COBS reader over std::io::Read.

use super::codec::cobs_decode;
use crate::config::RelayError;
use crate::telemetry::{PlatformPose, TelemetryIngest};
use std::io::{ErrorKind, Read};
use stm32_navcore::protocol::NavTelemetry;

pub struct SerialCobsReader<R> {
    pub reader: R,
    rx_buf: Vec<u8>,
    ingest: TelemetryIngest,
    max_buf_len: usize,
}

impl<R: Read> SerialCobsReader<R> {
    pub const DEFAULT_MAX_BUF: usize = 2048;

    pub fn new(reader: R, ingest: TelemetryIngest) -> Self {
        Self {
            reader,
            rx_buf: Vec::with_capacity(256),
            ingest,
            max_buf_len: Self::DEFAULT_MAX_BUF,
        }
    }

    pub fn with_max_buf(mut self, max: usize) -> Self {
        self.max_buf_len = max;
        self
    }

    pub fn ingest(&self) -> &TelemetryIngest {
        &self.ingest
    }

    pub fn ingest_mut(&mut self) -> &mut TelemetryIngest {
        &mut self.ingest
    }

    pub fn poll_next_pose(&mut self) -> Result<Option<PlatformPose>, RelayError> {
        let mut chunk = [0u8; 256];
        loop {
            match self.reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    self.rx_buf.extend_from_slice(&chunk[..n]);
                    if self.rx_buf.len() > self.max_buf_len {
                        self.rx_buf.drain(..self.rx_buf.len() - self.max_buf_len);
                    }
                }
                Err(ref e)
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                {
                    break
                }
                Err(e) => return Err(RelayError::IoError(e.to_string())),
            }
        }

        while let Some(pos) = self.rx_buf.iter().position(|&b| b == 0x00) {
            let framed: Vec<u8> = self.rx_buf.drain(..=pos).collect();
            let cobs_payload = &framed[..framed.len() - 1];
            if cobs_payload.is_empty() {
                continue;
            }
            if let Ok(decoded) = cobs_decode(cobs_payload) {
                if let Ok(telem) = NavTelemetry::unpack_uart(&decoded) {
                    let pose = self.ingest.update_from_nav(&telem);
                    return Ok(Some(pose));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::cobs::codec::cobs_encode;
    use std::io::Cursor;

    #[test]
    fn test_serial_cobs_reader_partial_stream() {
        let telem = NavTelemetry {
            timestamp_ms: 12345,
            pos_ned_m: [100.0, -50.0, -20.0],
            vel_ned_mps: [15.0, 0.0, -1.0],
            euler_rad: [0.05, -0.02, 1.57],
            status: 1,
        };
        let mut raw_uart = [0u8; 64];
        let raw_len = telem.pack_uart(&mut raw_uart).expect("pack uart");
        let mut framed = cobs_encode(&raw_uart[..raw_len]);
        framed.push(0x00);

        let mid = framed.len() / 2;
        let mut stream_bytes = Vec::new();
        stream_bytes.extend_from_slice(&framed[..mid]);
        let cursor = Cursor::new(stream_bytes);
        let ingest = TelemetryIngest::new(0.0, 0.0, 0.0, 5000);
        let mut reader = SerialCobsReader::new(cursor, ingest);

        assert!(reader.poll_next_pose().unwrap().is_none());

        let mut remaining = Vec::new();
        remaining.extend_from_slice(&framed[mid..]);
        remaining.extend_from_slice(b"\x05\x01\x02\x00");
        let cursor2 = Cursor::new(remaining);
        reader.reader = cursor2;

        let pose = reader.poll_next_pose().unwrap().expect("pose received");
        assert_eq!(pose.timestamp_ms, 12345);
        assert!((pose.yaw_rad - 1.57).abs() < 1e-4);
    }
}
