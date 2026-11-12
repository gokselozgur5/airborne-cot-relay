use airborne_cot_relay::RelayConfig;

fn simulate_step(step: u32, lat: &mut f64, lon: &mut f64) {
    *lat += 0.0001;
    *lon += 0.0001;
    println!("Step {step:>2}: Platform pose lat={lat:.4}, lon={lon:.4}");
}
