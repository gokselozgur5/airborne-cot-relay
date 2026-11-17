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

impl fmt::Display for FragmentationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PacketTooShort => write!(f, "packet too short for header"),
            Self::InvalidMagic => write!(f, "invalid magic header bytes"),
            Self::UnsupportedVersion => write!(f, "unsupported protocol version"),
            Self::InvalidTotalFragments => write!(f, "total fragments cannot be zero"),
            Self::IndexOutOfBounds => write!(f, "fragment index exceeds total fragments"),
            Self::MtuTooSmall => write!(f, "configured MTU is too small"),
            Self::ChecksumMismatch => write!(f, "fragment payload checksum mismatch"),
            Self::CapacityExceeded => write!(f, "maximum concurrent reassembly sessions exceeded"),
            Self::PayloadTooLarge => write!(f, "payload exceeds maximum supported size"),
        }
    }
}

/// Compute standard CRC-16-CCITT for payload integrity.
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// 16-byte protocol header for payload fragments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FragmentHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub flags: u8,
    pub msg_id: u32,
    pub seq_index: u16,
    pub total_fragments: u16,
    pub checksum: u16,
}

impl FragmentHeader {
    pub fn new(
        msg_id: u32,
        seq_index: u16,
        total_fragments: u16,
        checksum: u16,
    ) -> Result<Self, FragmentationError> {
        if total_fragments == 0 {
            return Err(FragmentationError::InvalidTotalFragments);
        }
        if seq_index >= total_fragments {
            return Err(FragmentationError::IndexOutOfBounds);
        }
        Ok(Self {
            magic: MAGIC_BYTES,
            version: PROTOCOL_VERSION,
            flags: 0,
            msg_id,
            seq_index,
            total_fragments,
            checksum,
        })
    }

