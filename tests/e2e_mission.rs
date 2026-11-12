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

#[test]
fn test_simulated_60_second_flight_mission() {
    let config = RelayConfig::default();
    assert!(config.validate().is_ok());
    let mut platform = SimulatedPlatform {
        lat_deg: 37.7749,
        lon_deg: -122.4194,
        altitude_m: 500.0,
        heading_deg: 90.0,
    };
    let vehicle = GroundTarget {
        id: 1,
        lat_deg: 37.7750,
        lon_deg: -122.4180,
        speed_mps: 15.0,
    };
    for _ in 0..60 {
        platform.step(1.0, 25.0);
    }
    assert!(platform.lon_deg > -122.4194);
    assert_eq!(vehicle.id, 1);
}
