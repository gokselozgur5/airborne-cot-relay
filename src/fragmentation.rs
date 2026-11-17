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
pub struct PayloadFragmenter {
    config: FragmenterConfig,
    next_msg_id: u32,
}

impl PayloadFragmenter {
    pub fn new(config: FragmenterConfig) -> Result<Self, FragmentationError> {
        if config.mtu < MIN_MTU {
            return Err(FragmentationError::MtuTooSmall);
        }
        Ok(Self {
            config,
            next_msg_id: 1,
        })
    }

    pub fn fragment(&mut self, payload: &[u8]) -> Result<Vec<FragmentPacket>, FragmentationError> {
        let msg_id = self.next_msg_id;
        self.next_msg_id = self.next_msg_id.wrapping_add(1);
        if self.next_msg_id == 0 {
            self.next_msg_id = 1;
        }
        self.fragment_with_id(msg_id, payload)
    }

    pub fn fragment_with_id(
        &self,
        msg_id: u32,
        payload: &[u8],
    ) -> Result<Vec<FragmentPacket>, FragmentationError> {
        if payload.len() > self.config.max_payload_size {
            return Err(FragmentationError::PayloadTooLarge);
        }
        let max_chunk = self.config.mtu - HEADER_SIZE;
        let total_fragments = if payload.is_empty() {
            1
        } else {
            (payload.len() + max_chunk - 1) / max_chunk
        };

        if total_fragments > u16::MAX as usize {
            return Err(FragmentationError::PayloadTooLarge);
        }

        let total_u16 = total_fragments as u16;
        let mut packets = Vec::with_capacity(total_fragments);

        if payload.is_empty() {
            let crc = crc16_ccitt(&[]);
            let header = FragmentHeader::new(msg_id, 0, 1, crc)?;
            packets.push(FragmentPacket::new(header, Vec::new()));
            return Ok(packets);
        }

        for (idx, chunk) in payload.chunks(max_chunk).enumerate() {
            let crc = crc16_ccitt(chunk);
            let header = FragmentHeader::new(msg_id, idx as u16, total_u16, crc)?;
            packets.push(FragmentPacket::new(header, chunk.to_vec()));
        }

        Ok(packets)
    }
}

/// Configuration settings for the reassembly engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReassemblyConfig {
    pub session_timeout_ms: u64,
    pub max_concurrent_sessions: usize,
}

impl Default for ReassemblyConfig {
    fn default() -> Self {
        Self {
            session_timeout_ms: 5000,
            max_concurrent_sessions: 128,
        }
    }
}

/// Tracking buffer for an in-flight fragmented message.
#[derive(Debug, Clone)]
pub struct InFlightSession {
    pub msg_id: u32,
    pub total_fragments: u16,
    pub received_fragments: BTreeMap<u16, Vec<u8>>,
    pub created_at_ms: u64,
    pub last_updated_ms: u64,
}

impl InFlightSession {
    pub fn new(msg_id: u32, total_fragments: u16, now_ms: u64) -> Self {
        Self {
            msg_id,
            total_fragments,
            received_fragments: BTreeMap::new(),
            created_at_ms: now_ms,
            last_updated_ms: now_ms,
        }
    }

    pub fn insert_fragment(&mut self, seq_index: u16, data: Vec<u8>, now_ms: u64) -> bool {
        self.last_updated_ms = now_ms;
        if self.received_fragments.contains_key(&seq_index) {
            return false;
        }
        self.received_fragments.insert(seq_index, data);
        true
    }

    pub fn is_complete(&self) -> bool {
        self.received_fragments.len() == self.total_fragments as usize
    }

    pub fn missing_indices(&self) -> Vec<u16> {
        let mut missing = Vec::new();
        for idx in 0..self.total_fragments {
            if !self.received_fragments.contains_key(&idx) {
                missing.push(idx);
            }
        }
        missing
    }

    pub fn assemble(self) -> Vec<u8> {
        let mut total_len = 0;
        for frag in self.received_fragments.values() {
            total_len += frag.len();
        }
        let mut payload = Vec::with_capacity(total_len);
        for (_, frag) in self.received_fragments {
            payload.extend_from_slice(&frag);
        }
        payload
    }
}

/// Diagnostic counters for the fragmentation and reassembly engine.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FragmentationStats {
    pub fragments_received: u64,
    pub messages_completed: u64,
    pub duplicate_fragments: u64,
    pub corrupt_fragments: u64,
    pub expired_sessions: u64,
}

/// Engine managing reassembly sessions for incoming fragmented UDP packets.
#[derive(Debug, Clone)]
pub struct ReassemblyEngine {
    config: ReassemblyConfig,
    sessions: HashMap<u32, InFlightSession>,
    stats: FragmentationStats,
}

impl ReassemblyEngine {
    pub fn new(config: ReassemblyConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            stats: FragmentationStats::default(),
        }
    }

    pub fn process_packet(
        &mut self,
        packet: &FragmentPacket,
        now_ms: u64,
    ) -> Result<Option<Vec<u8>>, FragmentationError> {
        self.stats.fragments_received += 1;
        let msg_id = packet.header.msg_id;
        let total = packet.header.total_fragments;
        let seq = packet.header.seq_index;

        if !self.sessions.contains_key(&msg_id) {
            if self.sessions.len() >= self.config.max_concurrent_sessions {
                return Err(FragmentationError::CapacityExceeded);
            }
            self.sessions
                .insert(msg_id, InFlightSession::new(msg_id, total, now_ms));
        }

        let session = self.sessions.get_mut(&msg_id).unwrap();
        if session.total_fragments != total {
            return Err(FragmentationError::InvalidTotalFragments);
        }

        let inserted = session.insert_fragment(seq, packet.data.clone(), now_ms);
        if !inserted {
            self.stats.duplicate_fragments += 1;
        }

        if session.is_complete() {
            let session = self.sessions.remove(&msg_id).unwrap();
            self.stats.messages_completed += 1;
            Ok(Some(session.assemble()))
        } else {
            Ok(None)
        }
    }

    pub fn process_bytes(
        &mut self,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<Option<Vec<u8>>, FragmentationError> {
        let packet = match FragmentPacket::from_bytes(bytes) {
            Ok(p) => p,
            Err(e) => {
                if e == FragmentationError::ChecksumMismatch {
                    self.stats.corrupt_fragments += 1;
                }
                return Err(e);
            }
        };
        self.process_packet(&packet, now_ms)
    }

    pub fn query_missing(&self, msg_id: u32) -> Option<Vec<u16>> {
        self.sessions.get(&msg_id).map(|s| s.missing_indices())
    }

    pub fn prune_expired(&mut self, now_ms: u64) -> usize {
        let timeout = self.config.session_timeout_ms;
        let initial_count = self.sessions.len();
        self.sessions
            .retain(|_, s| now_ms.saturating_sub(s.last_updated_ms) <= timeout);
        let pruned = initial_count - self.sessions.len();
        self.stats.expired_sessions += pruned as u64;
        pruned
    }

    pub fn stats(&self) -> &FragmentationStats {
        &self.stats
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_encode_decode() {
        let header = FragmentHeader::new(42, 1, 3, 0xABCD).unwrap();
        let bytes = header.encode();
        assert_eq!(bytes.len(), HEADER_SIZE);
        let decoded = FragmentHeader::decode(&bytes).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_crc16_integrity() {
        let data1 = b"test payload 1";
        let data2 = b"test payload 2";
        let crc1 = crc16_ccitt(data1);
        let crc2 = crc16_ccitt(data2);
        assert_ne!(crc1, crc2);
        assert_eq!(crc1, crc16_ccitt(data1));
    }

    #[test]
    fn test_packet_from_bytes_crc_mismatch() {
        let header = FragmentHeader::new(10, 0, 1, 0x1234).unwrap();
        let mut raw = header.encode().to_vec();
        raw.extend_from_slice(b"some data");
        let result = FragmentPacket::from_bytes(&raw);
        assert_eq!(result, Err(FragmentationError::ChecksumMismatch));
    }

    #[test]
    fn test_header_validation_errors() {
        assert_eq!(
            FragmentHeader::new(1, 0, 0, 0),
            Err(FragmentationError::InvalidTotalFragments)
        );
        assert_eq!(
            FragmentHeader::new(1, 3, 3, 0),
            Err(FragmentationError::IndexOutOfBounds)
        );
    }

    #[test]
    fn test_fragmentation_single_packet() {
        let mut fragmenter = PayloadFragmenter::new(FragmenterConfig::default()).unwrap();
        let payload = b"<event version='2.0' uid='drone-1'/>";
        let fragments = fragmenter.fragment(payload).unwrap();

        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].data, payload);

        let mut engine = ReassemblyEngine::new(ReassemblyConfig::default());
        let result = engine.process_packet(&fragments[0], 1000).unwrap();
        assert_eq!(result, Some(payload.to_vec()));
    }

    #[test]
    fn test_empty_payload() {
        let mut fragmenter = PayloadFragmenter::new(FragmenterConfig::default()).unwrap();
        let fragments = fragmenter.fragment(b"").unwrap();
        assert_eq!(fragments.len(), 1);
        assert!(fragments[0].data.is_empty());
    }

    #[test]
    fn test_multi_fragment_out_of_order() {
        let config = FragmenterConfig {
            mtu: 32,
            max_payload_size: 1024,
        };
        let mut fragmenter = PayloadFragmenter::new(config).unwrap();
        let payload = vec![0x42; 50];
        let fragments = fragmenter.fragment(&payload).unwrap();
        assert_eq!(fragments.len(), 4);

        let mut engine = ReassemblyEngine::new(ReassemblyConfig::default());
        assert_eq!(engine.process_packet(&fragments[2], 100).unwrap(), None);
        assert_eq!(engine.process_packet(&fragments[0], 110).unwrap(), None);
        assert_eq!(engine.process_packet(&fragments[0], 120).unwrap(), None);
        assert_eq!(engine.process_packet(&fragments[3], 130).unwrap(), None);
        let completed = engine.process_packet(&fragments[1], 140).unwrap();
        assert_eq!(completed, Some(payload));
        assert_eq!(engine.stats().duplicate_fragments, 1);
    }

    #[test]
    fn test_missing_query_and_timeout() {
        let config = FragmenterConfig {
            mtu: 32,
            max_payload_size: 1024,
        };
        let mut fragmenter = PayloadFragmenter::new(config).unwrap();
        let payload = vec![0x77; 40];
        let fragments = fragmenter.fragment(&payload).unwrap();

        let mut engine = ReassemblyEngine::new(ReassemblyConfig {
            session_timeout_ms: 500,
            max_concurrent_sessions: 2,
        });

        engine.process_packet(&fragments[0], 1000).unwrap();
        let missing = engine.query_missing(fragments[0].header.msg_id).unwrap();
        assert_eq!(missing, vec![1, 2]);

        assert_eq!(engine.prune_expired(1200), 0);
        assert_eq!(engine.prune_expired(1600), 1);
        assert_eq!(engine.active_session_count(), 0);
        assert_eq!(engine.stats().expired_sessions, 1);
    }
}
