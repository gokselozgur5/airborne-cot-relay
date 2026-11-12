/// Converts an ENU velocity to a compass course in degrees.
///
/// The result is clockwise from true north in `[0, 360)`. A zero (or
/// negative zero) velocity has no direction and yields `0.0`.
pub fn course_deg(v_east: f64, v_north: f64) -> f64 {
    if v_east == 0.0 && v_north == 0.0 {
        return 0.0;
    }
    let course = v_east.atan2(v_north).to_degrees().rem_euclid(360.0) + 0.0;
    if course >= 360.0 {
        0.0
    } else {
        course
    }
}
