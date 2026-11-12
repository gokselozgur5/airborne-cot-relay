use airborne_cot_relay::bearing::course_deg;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn cardinal_directions() {
    assert!(close(course_deg(0.0, 1.0), 0.0));
    assert!(close(course_deg(1.0, 0.0), 90.0));
    assert!(close(course_deg(0.0, -1.0), 180.0));
    assert!(close(course_deg(-1.0, 0.0), 270.0));
}

#[test]
fn diagonals() {
    assert!(close(course_deg(1.0, 1.0), 45.0));
    assert!(close(course_deg(1.0, -1.0), 135.0));
    assert!(close(course_deg(-1.0, -1.0), 225.0));
    assert!(close(course_deg(-1.0, 1.0), 315.0));
}

#[test]
fn zero_and_negative_zero_yield_zero() {
    assert_eq!(course_deg(0.0, 0.0), 0.0);
    assert_eq!(course_deg(-0.0, -0.0), 0.0);
    assert!(course_deg(-0.0, 1.0).is_sign_positive());
}

#[test]
fn result_is_always_below_360() {
    assert!(course_deg(-1e-20, 1.0) < 360.0);
}
