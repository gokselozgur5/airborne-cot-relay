use airborne_cot_relay::TelemetryIngest;

/// WGS84 radii of curvature at latitude 0: M = a(1 - e^2) = 6335439.327 m,
/// N = a = 6378137.0 m. 10 km north is 0.0904369 deg, 10 km east is 0.0898315 deg.
#[test]
fn ned_offset_of_10_km_matches_wgs84_within_5_m() {
    let ingest = TelemetryIngest::new(0.0, 0.0, 0.0, 5000);
    let (lat, lon, _) = ingest.ned_to_geodetic([10_000.0, 10_000.0, 0.0]);
    let north_err_m = (lat - 0.0904369) * 110_574.3;
    let east_err_m = (lon - 0.0898315) * 111_319.5;
    assert!(
        north_err_m.abs() < 5.0 && east_err_m.abs() < 5.0,
        "north error {north_err_m:.1} m, east error {east_err_m:.1} m"
    );
}
