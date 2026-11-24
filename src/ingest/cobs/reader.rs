//! Streaming COBS reader over std::io::Read.

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
    pub const DEFAULT_MAX_BUF: usize = 4096;

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

    fn extract_buffered_pose(&mut self) -> Option<PlatformPose> {
        while let Some(pos) = self.rx_buf.iter().position(|&b| b == 0x00) {
            let frame: Vec<u8> = self.rx_buf.drain(..=pos).collect();
            if frame.len() > 1 {
                if let Ok(telem) = NavTelemetry::unpack_uart(&frame) {
                    let pose = self.ingest.update_from_nav(&telem);
                    return Some(pose);
                }
            }
        }
        None
    }

    pub fn poll_next_pose(&mut self) -> Result<Option<PlatformPose>, RelayError> {
        if let Some(pose) = self.extract_buffered_pose() {
            return Ok(Some(pose));
        }

        let mut chunk = [0u8; 256];
        loop {
            match self.reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    self.rx_buf.extend_from_slice(&chunk[..n]);
                    if self.rx_buf.len() > self.max_buf_len {
                        let excess = self.rx_buf.len() - self.max_buf_len;
                        self.rx_buf.drain(..excess);
                    }
                    if let Some(pose) = self.extract_buffered_pose() {
                        return Ok(Some(pose));
                    }
                }
                Err(ref e)
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(e) => return Err(RelayError::IoError(e.to_string())),
            }
        }

        Ok(self.extract_buffered_pose())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let framed = &raw_uart[..raw_len];

        let mid = framed.len() / 2;
        let mut stream_bytes = Vec::new();
        stream_bytes.extend_from_slice(&framed[..mid]);
        let cursor = Cursor::new(stream_bytes);
        let ingest = TelemetryIngest::new(0.0, 0.0, 0.0, 5000);
        let mut reader = SerialCobsReader::new(cursor, ingest);

        assert!(reader.poll_next_pose().unwrap().is_none());

        let mut remaining = Vec::new();
        remaining.extend_from_slice(&framed[mid..]);
        let cursor2 = Cursor::new(remaining);
        reader.reader = cursor2;

        let pose = reader.poll_next_pose().unwrap().expect("pose received");
        assert_eq!(pose.timestamp_ms, 12345);
        assert!((pose.yaw_rad - 1.57).abs() < 1e-4);
    }

    #[test]
    fn test_serial_cobs_reader_back_to_back_frames() {
        let telem1 = NavTelemetry {
            timestamp_ms: 1000,
            pos_ned_m: [10.0, 20.0, -30.0],
            vel_ned_mps: [1.0, 2.0, 3.0],
            euler_rad: [0.1, 0.2, 0.3],
            status: 1,
        };
        let telem2 = NavTelemetry {
            timestamp_ms: 2000,
            pos_ned_m: [40.0, 50.0, -60.0],
            vel_ned_mps: [4.0, 5.0, 6.0],
            euler_rad: [0.4, 0.5, 0.6],
            status: 1,
        };
        let mut b1 = [0u8; 64];
        let n1 = telem1.pack_uart(&mut b1).expect("pack 1");
        let mut b2 = [0u8; 64];
        let n2 = telem2.pack_uart(&mut b2).expect("pack 2");

        let mut stream = Vec::new();
        stream.extend_from_slice(&b1[..n1]);
        stream.extend_from_slice(&b2[..n2]);

        let cursor = Cursor::new(stream);
        let ingest = TelemetryIngest::new(0.0, 0.0, 0.0, 5000);
        let mut reader = SerialCobsReader::new(cursor, ingest);

        let p1 = reader.poll_next_pose().unwrap().expect("p1");
        assert_eq!(p1.timestamp_ms, 1000);

        // Second poll should yield telem2 directly from buffer without needing more data
        let p2 = reader.poll_next_pose().unwrap().expect("p2");
        assert_eq!(p2.timestamp_ms, 2000);
    }
}
