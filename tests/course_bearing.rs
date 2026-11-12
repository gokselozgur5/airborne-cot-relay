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

