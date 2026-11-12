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
