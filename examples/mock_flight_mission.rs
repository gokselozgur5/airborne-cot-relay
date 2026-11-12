use airborne_cot_relay::RelayConfig;

fn simulate_step(step: u32, lat: &mut f64, lon: &mut f64) {
    *lat += 0.0001;
    *lon += 0.0001;
    println!("Step {step:>2}: Platform pose lat={lat:.4}, lon={lon:.4}");
}

fn main() {
    let cfg = RelayConfig::default();
    println!("Running mock flight mission with config: {cfg:?}");
    let mut lat = 37.7749;
    let mut lon = -122.4194;
    for step in 1..=5 {
        simulate_step(step, &mut lat, &mut lon);
    }
    println!("Mock flight mission complete.");
}
