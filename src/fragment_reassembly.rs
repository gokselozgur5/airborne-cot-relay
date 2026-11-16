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
