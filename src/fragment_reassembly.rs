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

    /// Serializes fragment to wire byte vector.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        buf.extend_from_slice(&self.header.encode());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Deserializes fragment from wire byte slice.
    pub fn decode(bytes: &[u8]) -> Result<Self, ReassemblyError> {
        let header = FragmentHeader::decode(bytes)?;
        let payload = bytes[HEADER_SIZE..].to_vec();
        Ok(Self { header, payload })
    }
}

/// Fragments a contiguous payload into MTU-bounded fragments.
pub fn fragment_payload(msg_id: u32, data: &[u8], mtu_size: usize) -> Result<Vec<Fragment>, ReassemblyError> {
    if mtu_size <= HEADER_SIZE {
        return Err(ReassemblyError::MtuTooSmall);
    }
    let max_chunk = mtu_size - HEADER_SIZE;
    if data.is_empty() {
        let header = FragmentHeader::new(msg_id, 0, 1)?;
        return Ok(vec![Fragment::new(header, Vec::new())]);
    }
    let chunks: Vec<&[u8]> = data.chunks(max_chunk).collect();
    let total_fragments = chunks.len() as u16;
    let mut fragments = Vec::with_capacity(chunks.len());
    for (idx, chunk) in chunks.into_iter().enumerate() {
        let header = FragmentHeader::new(msg_id, idx as u16, total_fragments)?;
        fragments.push(Fragment::new(header, chunk.to_vec()));
    }
    Ok(fragments)
}

/// Telemetry statistics for fragment reassembly operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReassemblyStats {
    pub packets_received: u64,
    pub messages_completed: u64,
    pub duplicate_fragments: u64,
    pub stale_dropped_messages: u64,
}

impl ReassemblyStats {
    /// Resets all counters to zero.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// In-flight message entry storing received fragment slots.
#[derive(Debug, Clone)]
struct InFlightMessage {
    total_fragments: u16,
    received_count: u16,
    slots: Vec<Option<Vec<u8>>>,
    created_at_ms: u64,
    last_updated_ms: u64,
}

impl InFlightMessage {
    fn new(total: u16, now_ms: u64) -> Self {
        Self {
            total_fragments: total,
            received_count: 0,
            slots: vec![None; total as usize],
            created_at_ms: now_ms,
            last_updated_ms: now_ms,
        }
    }
}
