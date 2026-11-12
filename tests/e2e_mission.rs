use airborne_cot_relay::RelayConfig;

#[derive(Debug)]
pub struct SimulatedPlatform {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub altitude_m: f64,
    pub heading_deg: f64,
}
