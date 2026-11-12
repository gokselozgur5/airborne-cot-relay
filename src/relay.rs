use crate::config::RelayError;
use sightline_cot::cot::{escape_xml_attr, format_iso8601_utc};
use sightline_cot::tracker::Track;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

#[derive(Debug, Clone, PartialEq)]
pub struct TrackSummary {
    pub id: u64,
    pub classification: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub hae_m: f64,
    pub ce90_m: f64,
    pub le90_m: f64,
    pub course_deg: f64,
    pub speed_mps: f64,
}

impl TrackSummary {
    pub fn from_track(
        track: &Track,
        classification: &str,
        origin_lat: f64,
        origin_lon: f64,
        origin_hae: f64,
    ) -> Self {
        let (lat_deg, lon_deg, hae_m) = sightline_cot::camera::enu_to_geodetic(
            [track.x[0], track.x[1], 0.0],
            origin_lat,
            origin_lon,
            origin_hae,
        );
        let speed_mps = (track.x[2] * track.x[2] + track.x[3] * track.x[3]).sqrt();
        let course_rad = track.x[0].atan2(track.x[1]);
        let course_deg = (course_rad.to_degrees() + 360.0) % 360.0;
        let ce90_m = 2.146 * track.p[0][0].max(track.p[1][1]).sqrt();
        let le90_m = 10.0;
        Self {
            id: track.id,
            classification: classification.to_string(),
            lat_deg,
            lon_deg,
            hae_m,
            ce90_m,
            le90_m,
            course_deg,
            speed_mps,
        }
    }
}

/// Bounded exponential backoff schedule for reconnect attempts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backoff {
    base_ms: u64,
    max_ms: u64,
    attempt: u32,
}

impl Backoff {
    /// Creates a schedule starting at `base_ms` and capped at `max_ms`.
    pub fn new(base_ms: u64, max_ms: u64) -> Self {
        let base_ms = base_ms.max(1);
        Self {
            base_ms,
            max_ms: max_ms.max(base_ms),
            attempt: 0,
        }
    }

    /// Returns the delay before the next attempt and advances the schedule.
    pub fn next_delay_ms(&mut self) -> u64 {
        let factor = 1u64.checked_shl(self.attempt).unwrap_or(u64::MAX);
        let delay = self.base_ms.saturating_mul(factor).min(self.max_ms);
        self.attempt = self.attempt.saturating_add(1);
        delay
    }

    /// Number of delays handed out since the last reset.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Restarts the schedule after a successful connection.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

pub fn classification_to_cot_type(classification: &str) -> &'static str {
    match classification.to_ascii_lowercase().as_str() {
        "vehicle" | "car" | "truck" => "a-f-G-E-V-C",
        "person" | "pedestrian" | "human" => "a-f-G-E-S",
        "aircraft" | "drone" | "uav" => "a-f-A-M-F",
        _ => "a-u-G",
    }
}

/// What a full queue does with an incoming item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPolicy {
    /// Evict the oldest queued item to make room for the new one.
    DropOldest,
    /// Reject the incoming item and keep the queue unchanged.
    DropNewest,
}

/// Fixed-capacity FIFO that never grows past its capacity.
#[derive(Debug)]
pub struct BoundedQueue<T> {
    items: std::collections::VecDeque<T>,
    capacity: usize,
    policy: DropPolicy,
    dropped: u64,
}

impl<T> BoundedQueue<T> {
    /// Creates a queue holding at most `capacity` items, minimum one.
    pub fn new(capacity: usize, policy: DropPolicy) -> Self {
        let capacity = capacity.max(1);
        Self {
            items: std::collections::VecDeque::with_capacity(capacity),
            capacity,
            policy,
            dropped: 0,
        }
    }

    /// Enqueues `item` and returns the item dropped when the queue was full.
    pub fn push(&mut self, item: T) -> Option<T> {
        if self.items.len() < self.capacity {
            self.items.push_back(item);
            return None;
        }
        self.dropped = self.dropped.saturating_add(1);
        match self.policy {
            DropPolicy::DropNewest => Some(item),
            DropPolicy::DropOldest => {
                let evicted = self.items.pop_front();
                self.items.push_back(item);
                evicted
            }
        }
    }

    /// Removes and returns the oldest queued item.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    /// Number of queued items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when nothing is queued.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Total items dropped since creation.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

pub struct CotStreamer {
    socket: UdpSocket,
    target_addr: SocketAddr,
    stale_timeout_ms: u64,
}

impl CotStreamer {
    pub fn new(
        bind_addr: &str,
        target_addr_str: &str,
        stale_timeout_ms: u64,
    ) -> Result<Self, RelayError> {
        let socket = UdpSocket::bind(bind_addr).map_err(|e| {
            RelayError::IoError(format!("failed to bind UDP socket {bind_addr}: {e}"))
        })?;
        socket
            .set_nonblocking(true)
            .map_err(|e| RelayError::IoError(format!("failed to set non-blocking: {e}")))?;
        let _ = socket.set_broadcast(true);
        let target_addr = target_addr_str
            .to_socket_addrs()
            .map_err(|e| {
                RelayError::IoError(format!("invalid target address {target_addr_str}: {e}"))
            })?
            .next()
            .ok_or_else(|| {
                RelayError::IoError(format!("no address resolved for {target_addr_str}"))
            })?;
        Ok(Self {
            socket,
            target_addr,
            stale_timeout_ms,
        })
    }

    pub fn build_cot_xml(&self, track: &TrackSummary, now_ms: i64) -> String {
        let cot_type = classification_to_cot_type(&track.classification);
        let stale_ms = now_ms + self.stale_timeout_ms as i64;
        let time_str = format_iso8601_utc(now_ms);
        let stale_str = format_iso8601_utc(stale_ms);
        let uid = format!("TRACK-{:04}", track.id);
        format!(
            r#"<event version="2.0" uid="{}" type="{}" how="m-g" time="{}" start="{}" stale="{}"><point lat="{:.7}" lon="{:.7}" hae="{:.2}" ce="{:.2}" le="{:.2}"/><detail><track course="{:.2}" speed="{:.2}"/></detail></event>"#,
            escape_xml_attr(&uid),
            escape_xml_attr(cot_type),
            time_str,
            time_str,
            stale_str,
            track.lat_deg,
            track.lon_deg,
            track.hae_m,
            track.ce90_m,
            track.le90_m,
            track.course_deg,
            track.speed_mps,
        )
    }

    pub fn send_track(&self, track: &TrackSummary, now_ms: i64) -> Result<usize, RelayError> {
        let xml = self.build_cot_xml(track, now_ms);
        self.send_xml(&xml)
    }

    pub fn send_xml(&self, xml: &str) -> Result<usize, RelayError> {
        self.socket
            .send_to(xml.as_bytes(), self.target_addr)
            .map_err(|e| RelayError::IoError(format!("UDP send failed: {e}")))
    }

    pub fn target_addr(&self) -> SocketAddr {
        self.target_addr
    }
}

/// Destination for encoded CoT datagrams, injectable for tests.
pub trait CotSink {
    /// Sends one encoded payload and returns the number of bytes written.
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError>;
}

impl CotSink for CotStreamer {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.socket
            .send_to(payload, self.target_addr)
            .map_err(|e| RelayError::IoError(format!("UDP send failed: {e}")))
    }
}

/// Tracks consecutive send failures and flags a sink that needs reconnecting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendHealth {
    max_consecutive_failures: u32,
    consecutive_failures: u32,
    total_failures: u64,
}

impl SendHealth {
    /// Creates a tracker that requests a reconnect after the given failures.
    pub fn new(max_consecutive_failures: u32) -> Self {
        Self {
            max_consecutive_failures: max_consecutive_failures.max(1),
            consecutive_failures: 0,
            total_failures: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sightline_cot::cot::CotEvent;

    #[test]
    fn test_classification_mapping() {
        assert_eq!(classification_to_cot_type("vehicle"), "a-f-G-E-V-C");
        assert_eq!(classification_to_cot_type("Car"), "a-f-G-E-V-C");
        assert_eq!(classification_to_cot_type("truck"), "a-f-G-E-V-C");
        assert_eq!(classification_to_cot_type("person"), "a-f-G-E-S");
        assert_eq!(classification_to_cot_type("pedestrian"), "a-f-G-E-S");
        assert_eq!(classification_to_cot_type("drone"), "a-f-A-M-F");
        assert_eq!(classification_to_cot_type("unknown_object"), "a-u-G");
    }

    #[test]
    fn test_backoff_doubles_then_caps() {
        let mut backoff = Backoff::new(100, 1000);
        let delays: Vec<u64> = (0..6).map(|_| backoff.next_delay_ms()).collect();
        assert_eq!(delays, vec![100, 200, 400, 800, 1000, 1000]);
        assert_eq!(backoff.attempt(), 6);
    }

    #[test]
    fn test_backoff_reset_and_clamping() {
        let mut backoff = Backoff::new(0, 0);
        assert_eq!(backoff.next_delay_ms(), 1);
        backoff.reset();
        assert_eq!(backoff.attempt(), 0);
        assert_eq!(backoff.next_delay_ms(), 1);
        let mut long = Backoff::new(50, u64::MAX);
        for _ in 0..200 {
            long.next_delay_ms();
        }
        assert_eq!(long.next_delay_ms(), u64::MAX);
    }

    #[test]
    fn test_cot_xml_generation_and_validation() {
        let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:6969", 5000).unwrap();
        let track = TrackSummary {
            id: 42,
            classification: "vehicle".to_string(),
            lat_deg: 37.7749,
            lon_deg: -122.4194,
            hae_m: 15.0,
            ce90_m: 3.5,
            le90_m: 10.0,
            course_deg: 90.0,
            speed_mps: 12.5,
        };
        let now_ms = 1731369600000;
        let xml = streamer.build_cot_xml(&track, now_ms);
        assert!(xml.contains(r#"uid="TRACK-0042""#));
        assert!(xml.contains(r#"type="a-f-G-E-V-C""#));
        assert!(xml.contains(r#"lat="37.7749000""#));
        let parsed = CotEvent::from_xml(&xml).expect("valid CoT 2.0 XML");
        assert_eq!(parsed.uid, "TRACK-0042");
        assert_eq!(parsed.cot_type, "a-f-G-E-V-C");
    }

    #[test]
    fn test_bounded_queue_drop_oldest() {
        let mut queue = BoundedQueue::new(2, DropPolicy::DropOldest);
        assert_eq!(queue.push(1), None);
        assert_eq!(queue.push(2), None);
        assert_eq!(queue.push(3), Some(1));
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.dropped(), 1);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_bounded_queue_drop_newest_and_clamp() {
        let mut queue = BoundedQueue::new(0, DropPolicy::DropNewest);
        assert_eq!(queue.push("a"), None);
        assert_eq!(queue.push("b"), Some("b"));
        assert_eq!(queue.push("c"), Some("c"));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.dropped(), 2);
        assert_eq!(queue.pop(), Some("a"));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_udp_socket_dispatch_loopback() {
        let rx_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        rx_sock
            .set_read_timeout(Some(std::time::Duration::from_millis(500)))
            .unwrap();
        let rx_port = rx_sock.local_addr().unwrap().port();
        let target = format!("127.0.0.1:{rx_port}");
        let streamer = CotStreamer::new("127.0.0.1:0", &target, 3000).unwrap();
        let track = TrackSummary {
            id: 1,
            classification: "person".to_string(),
            lat_deg: 36.0,
            lon_deg: 32.0,
            hae_m: 100.0,
            ce90_m: 2.0,
            le90_m: 5.0,
            course_deg: 0.0,
            speed_mps: 1.5,
        };
        let sent = streamer.send_track(&track, 1731369600000).unwrap();
        assert!(sent > 0);
        let mut buf = [0u8; 1024];
        let (len, _) = rx_sock.recv_from(&mut buf).unwrap();
        let received_xml = std::str::from_utf8(&buf[..len]).unwrap();
        assert!(received_xml.contains(r#"uid="TRACK-0001""#));
    }
}
