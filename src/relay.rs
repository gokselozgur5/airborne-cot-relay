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
