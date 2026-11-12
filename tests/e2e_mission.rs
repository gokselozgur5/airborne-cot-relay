use airborne_cot_relay::RelayConfig;

#[derive(Debug)]
pub struct SimulatedPlatform {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub altitude_m: f64,
    pub heading_deg: f64,
}

impl SimulatedPlatform {
    pub fn step(&mut self, dt_sec: f64, ground_speed_mps: f64) {
        let lat_rad = self.lat_deg.to_radians();
        let d_north = ground_speed_mps * dt_sec * self.heading_deg.to_radians().cos();
        let d_east = ground_speed_mps * dt_sec * self.heading_deg.to_radians().sin();
        self.lat_deg += (d_north / 6378137.0).to_degrees();
        self.lon_deg += (d_east / (6378137.0 * lat_rad.cos())).to_degrees();
    }
}

#[derive(Debug)]
pub struct GroundTarget {
    pub id: u32,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub speed_mps: f64,
}
