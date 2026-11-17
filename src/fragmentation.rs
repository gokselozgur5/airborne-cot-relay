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

    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4] = self.version;
        buf[5] = self.flags;
        buf[6..10].copy_from_slice(&self.msg_id.to_be_bytes());
        buf[10..12].copy_from_slice(&self.seq_index.to_be_bytes());
        buf[12..14].copy_from_slice(&self.total_fragments.to_be_bytes());
        buf[14..16].copy_from_slice(&self.checksum.to_be_bytes());
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, FragmentationError> {
        if bytes.len() < HEADER_SIZE {
            return Err(FragmentationError::PacketTooShort);
        }
        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if magic != MAGIC_BYTES {
            return Err(FragmentationError::InvalidMagic);
        }
        let version = bytes[4];
        if version != PROTOCOL_VERSION {
            return Err(FragmentationError::UnsupportedVersion);
        }
        let flags = bytes[5];
        let msg_id = u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        let seq_index = u16::from_be_bytes([bytes[10], bytes[11]]);
        let total_fragments = u16::from_be_bytes([bytes[12], bytes[13]]);
        let checksum = u16::from_be_bytes([bytes[14], bytes[15]]);

        if total_fragments == 0 {
            return Err(FragmentationError::InvalidTotalFragments);
        }
        if seq_index >= total_fragments {
            return Err(FragmentationError::IndexOutOfBounds);
        }

        Ok(Self {
            magic,
            version,
            flags,
            msg_id,
            seq_index,
            total_fragments,
            checksum,
        })
    }
}

/// Encapsulates a fragment header and partial payload data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentPacket {
    pub header: FragmentHeader,
    pub data: Vec<u8>,
}

impl FragmentPacket {
    pub fn new(header: FragmentHeader, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_SIZE + self.data.len());
        out.extend_from_slice(&self.header.encode());
        out.extend_from_slice(&self.data);
        out
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Self, FragmentationError> {
        if raw.len() < HEADER_SIZE {
            return Err(FragmentationError::PacketTooShort);
        }
        let header = FragmentHeader::decode(raw)?;
        let data = raw[HEADER_SIZE..].to_vec();
        let expected_crc = crc16_ccitt(&data);
        if header.checksum != expected_crc {
            return Err(FragmentationError::ChecksumMismatch);
        }
        Ok(Self { header, data })
    }
}

/// Configuration settings for the payload fragmenter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmenterConfig {
    pub mtu: usize,
    pub max_payload_size: usize,
}

impl Default for FragmenterConfig {
    fn default() -> Self {
        Self {
            mtu: DEFAULT_MTU,
            max_payload_size: 1024 * 1024,
        }
    }
}

/// Engine responsible for fragmenting large CoT payloads into MTU-compliant packets.
#[derive(Debug, Clone)]
