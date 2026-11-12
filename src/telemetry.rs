use stm32_navcore::protocol::NavTelemetry;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformPose {
    pub timestamp_ms: u64,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub hae_m: f64,
    pub roll_rad: f64,
    pub pitch_rad: f64,
    pub yaw_rad: f64,
}

impl PlatformPose {
    pub const fn new(
        timestamp_ms: u64,
        lat_deg: f64,
        lon_deg: f64,
        hae_m: f64,
        roll_rad: f64,
        pitch_rad: f64,
        yaw_rad: f64,
    ) -> Self {
        Self {
            timestamp_ms,
            lat_deg,
            lon_deg,
            hae_m,
            roll_rad,
            pitch_rad,
            yaw_rad,
        }
    }
}

impl PlatformPose {
    pub fn is_stale(&self, now_ms: u64, max_age_ms: u64) -> bool {
        now_ms.saturating_sub(self.timestamp_ms) > max_age_ms
    }
}

pub struct TelemetryIngest {
    rx_buf: Vec<u8>,
    origin_lat_deg: f64,
    origin_lon_deg: f64,
    origin_alt_m: f64,
    max_age_ms: u64,
    latest_pose: Option<PlatformPose>,
}

impl TelemetryIngest {
    pub fn new(
        origin_lat_deg: f64,
        origin_lon_deg: f64,
        origin_alt_m: f64,
        max_age_ms: u64,
    ) -> Self {
        Self {
            rx_buf: Vec::with_capacity(256),
            origin_lat_deg,
            origin_lon_deg,
            origin_alt_m,
            max_age_ms,
            latest_pose: None,
        }
    }
}
