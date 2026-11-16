//! UDP payload MTU fragmentation reassembler for Cursor-on-Target telemetry.

use std::collections::HashMap;
use std::fmt;

/// Errors produced during fragment encoding, decoding, or reassembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReassemblyError {
    PacketTooShort,
    InvalidTotalFragments,
    IndexOutOfBounds,
    MtuTooSmall,
    CapacityExceeded,
    PayloadMismatch,
}

impl std::error::Error for ReassemblyError {}

impl fmt::Display for ReassemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PacketTooShort => write!(f, "packet too short for header"),
            Self::InvalidTotalFragments => write!(f, "total fragments cannot be zero"),
            Self::IndexOutOfBounds => write!(f, "fragment index out of bounds"),
            Self::MtuTooSmall => write!(f, "MTU too small to hold payload"),
            Self::CapacityExceeded => write!(f, "reassembler capacity exceeded"),
            Self::PayloadMismatch => write!(f, "payload length mismatch"),
        }
    }
}

/// Binary header size in bytes: 4 (msg_id) + 2 (seq_index) + 2 (total_fragments).
pub const HEADER_SIZE: usize = 8;

/// 8-byte header preceding each fragmented UDP packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FragmentHeader {
    pub msg_id: u32,
    pub seq_index: u16,
    pub total_fragments: u16,
}

impl FragmentHeader {
    /// Creates a new fragment header.
    pub fn new(msg_id: u32, seq_index: u16, total_fragments: u16) -> Result<Self, ReassemblyError> {
        if total_fragments == 0 {
            return Err(ReassemblyError::InvalidTotalFragments);
        }
        if seq_index >= total_fragments {
            return Err(ReassemblyError::IndexOutOfBounds);
        }
        Ok(Self { msg_id, seq_index, total_fragments })
    }

    /// Encodes header into an 8-byte big-endian array.
    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.msg_id.to_be_bytes());
        buf[4..6].copy_from_slice(&self.seq_index.to_be_bytes());
        buf[6..8].copy_from_slice(&self.total_fragments.to_be_bytes());
        buf
    }

    /// Decodes header from an 8-byte big-endian slice.
    pub fn decode(bytes: &[u8]) -> Result<Self, ReassemblyError> {
        if bytes.len() < HEADER_SIZE {
            return Err(ReassemblyError::PacketTooShort);
        }
        let msg_id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let seq_index = u16::from_be_bytes([bytes[4], bytes[5]]);
        let total_fragments = u16::from_be_bytes([bytes[6], bytes[7]]);
        Self::new(msg_id, seq_index, total_fragments)
    }
}

/// Complete fragment containing header and partial payload bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub header: FragmentHeader,
    pub payload: Vec<u8>,
}

impl Fragment {
    /// Creates a new fragment from header and payload.
    pub fn new(header: FragmentHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// Returns the fragment byte length including the 8-byte header.
    pub fn wire_size(&self) -> usize {
        HEADER_SIZE + self.payload.len()
    }
}
