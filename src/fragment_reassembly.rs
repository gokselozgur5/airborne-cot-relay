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
