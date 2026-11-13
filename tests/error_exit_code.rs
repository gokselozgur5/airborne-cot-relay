use airborne_cot_relay::error::exit::exit_code;
use airborne_cot_relay::RelayError;

fn all_variants() -> [RelayError; 4] {
    [
        RelayError::InvalidConfig("c".into()),
        RelayError::IoError("i".into()),
        RelayError::DecodeError("d".into()),
        RelayError::TargetingError("t".into()),
    ]
}

#[test]
fn each_variant_has_documented_code() {
    let codes: Vec<i32> = all_variants().iter().map(exit_code).collect();
    assert_eq!(codes, vec![2, 3, 4, 5]);
}

#[test]
fn codes_are_distinct_and_non_zero() {
    let mut codes: Vec<i32> = all_variants().iter().map(exit_code).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), 4);
    assert!(codes.iter().all(|c| *c != 0));
}
