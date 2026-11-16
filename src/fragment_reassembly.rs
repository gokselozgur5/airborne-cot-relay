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
}
