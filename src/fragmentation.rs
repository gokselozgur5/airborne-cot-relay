//! CoT payload fragmentation and reassembly protocol engine for MTU limits.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

/// Magic bytes identifying a fragmentation frame ("CFRG").
pub const MAGIC_BYTES: [u8; 4] = [0x43, 0x46, 0x52, 0x47];

/// Current fragmentation protocol version.
pub const PROTOCOL_VERSION: u8 = 1;

/// Binary header size in bytes: 16.
pub const HEADER_SIZE: usize = 16;

/// Minimum allowable MTU.
pub const MIN_MTU: usize = HEADER_SIZE + 1;

/// Default MTU for UDP payloads over standard links.
pub const DEFAULT_MTU: usize = 1400;

/// Errors produced during fragmentation and reassembly operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentationError {
    PacketTooShort,
    InvalidMagic,
    UnsupportedVersion,
    InvalidTotalFragments,
    IndexOutOfBounds,
    MtuTooSmall,
    ChecksumMismatch,
    CapacityExceeded,
    PayloadTooLarge,
}

impl std::error::Error for FragmentationError {}

