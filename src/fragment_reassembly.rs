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
    last_updated_ms: u64,
}

impl InFlightMessage {
    fn new(total: u16, now_ms: u64) -> Self {
        Self {
            total_fragments: total,
            received_count: 0,
            slots: vec![None; total as usize],
            last_updated_ms: now_ms,
        }
    }

    fn assemble(&self) -> Vec<u8> {
        let total_size: usize = self.slots.iter().map(|s| s.as_ref().map_or(0, |b| b.len())).sum();
        let mut assembled = Vec::with_capacity(total_size);
        for slot in &self.slots {
            if let Some(chunk) = slot {
                assembled.extend_from_slice(chunk);
            }
        }
        assembled
    }

    fn insert(&mut self, seq: usize, payload: Vec<u8>) -> bool {
        if seq < self.slots.len() && self.slots[seq].is_none() {
            self.slots[seq] = Some(payload);
            self.received_count += 1;
            true
        } else {
            false
        }
    }

    fn is_complete(&self) -> bool {
        self.received_count == self.total_fragments
    }
}

/// Lock-free/deterministic UDP fragment reassembler.
#[derive(Debug)]
pub struct FragmentReassembler {
    in_flight: HashMap<u32, InFlightMessage>,
    max_in_flight: usize,
    ttl_ms: u64,
    stats: ReassemblyStats,
}

impl Default for FragmentReassembler {
    fn default() -> Self {
        Self::new(256, 5000)
    }
}

impl FragmentReassembler {
    /// Creates a reassembler with max concurrent in-flight messages and TTL in ms.
    pub fn new(max_in_flight: usize, ttl_ms: u64) -> Self {
        Self {
            in_flight: HashMap::new(),
            max_in_flight: max_in_flight.max(1),
            ttl_ms: ttl_ms.max(1),
            stats: ReassemblyStats::default(),
        }
    }

    /// Returns the current number of in-flight incomplete messages.
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Returns a copy of the current reassembly statistics.
    pub fn stats(&self) -> ReassemblyStats {
        self.stats
    }

    /// Prunes in-flight messages that have exceeded their TTL.
    pub fn prune_stale(&mut self, now_ms: u64) -> usize {
        let ttl = self.ttl_ms;
        let mut expired = Vec::new();
        for (&id, msg) in &self.in_flight {
            if now_ms.saturating_sub(msg.last_updated_ms) > ttl {
                expired.push(id);
            }
        }
        let count = expired.len();
        for id in expired {
            self.in_flight.remove(&id);
            self.stats.stale_dropped_messages += 1;
        }
        count
    }

    fn check_capacity(&mut self, id: u32, now_ms: u64) -> Result<(), ReassemblyError> {
        if !self.in_flight.contains_key(&id) && self.in_flight.len() >= self.max_in_flight {
            self.prune_stale(now_ms);
            if self.in_flight.len() >= self.max_in_flight {
                return Err(ReassemblyError::CapacityExceeded);
            }
        }
        Ok(())
    }

    /// Ingests a decoded fragment. Returns complete payload when all parts arrive.
    pub fn process_fragment(&mut self, frag: Fragment, now_ms: u64) -> Result<Option<Vec<u8>>, ReassemblyError> {
        self.stats.packets_received += 1;
        if frag.header.total_fragments == 1 {
            self.stats.messages_completed += 1;
            return Ok(Some(frag.payload));
        }
        let (id, total, seq) = (frag.header.msg_id, frag.header.total_fragments, frag.header.seq_index as usize);
        self.check_capacity(id, now_ms)?;
        let entry = self.in_flight.entry(id).or_insert_with(|| InFlightMessage::new(total, now_ms));
        if entry.total_fragments != total {
            return Err(ReassemblyError::PayloadMismatch);
        }
        entry.last_updated_ms = now_ms;
        if !entry.insert(seq, frag.payload) {
            self.stats.duplicate_fragments += 1;
        }
        if entry.is_complete() {
            let completed = self.in_flight.remove(&id).unwrap();
            self.stats.messages_completed += 1;
            Ok(Some(completed.assemble()))
        } else {
            Ok(None)
        }
    }

    /// Ingests raw packet bytes directly, decoding header and payload.
    pub fn process_packet(&mut self, packet_bytes: &[u8], now_ms: u64) -> Result<Option<Vec<u8>>, ReassemblyError> {
        let frag = Fragment::decode(packet_bytes)?;
        self.process_fragment(frag, now_ms)
    }

    /// Clears all in-flight buffers and resets statistics.
    pub fn clear(&mut self) {
        self.in_flight.clear();
        self.stats.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_header_codec() {
        let header = FragmentHeader::new(0x12345678, 2, 5).unwrap();
        assert_eq!(header.msg_id, 0x12345678);
        assert_eq!(header.seq_index, 2);
        assert_eq!(header.total_fragments, 5);
        let encoded = header.encode();
        assert_eq!(encoded.len(), HEADER_SIZE);
        let decoded = FragmentHeader::decode(&encoded).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_fragment_codec_and_validation() {
        let header = FragmentHeader::new(42, 0, 1).unwrap();
        let frag = Fragment::new(header, b"<event/>".to_vec());
        assert_eq!(frag.wire_size(), HEADER_SIZE + 8);
        let encoded = frag.encode();
        let decoded = Fragment::decode(&encoded).unwrap();
        assert_eq!(frag, decoded);
        assert_eq!(FragmentHeader::new(1, 0, 0), Err(ReassemblyError::InvalidTotalFragments));
        assert_eq!(FragmentHeader::new(1, 3, 2), Err(ReassemblyError::IndexOutOfBounds));
        assert_eq!(FragmentHeader::decode(&[0; 4]), Err(ReassemblyError::PacketTooShort));
    }
}
