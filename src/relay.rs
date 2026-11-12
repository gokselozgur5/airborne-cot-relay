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

pub fn classification_to_cot_type(classification: &str) -> &'static str {
    match classification.to_ascii_lowercase().as_str() {
        "vehicle" | "car" | "truck" => "a-f-G-E-V-C",
        "person" | "pedestrian" | "human" => "a-f-G-E-S",
        "aircraft" | "drone" | "uav" => "a-f-A-M-F",
        _ => "a-u-G",
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
