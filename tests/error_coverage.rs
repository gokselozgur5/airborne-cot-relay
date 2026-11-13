use airborne_cot_relay::RelayError;

/// One value per variant; the exhaustive match fails to compile on a new one.
#[allow(dead_code)]
fn all_variants(msg: &str) -> Vec<(RelayError, &'static str)> {
    let list = vec![
        (RelayError::InvalidConfig(msg.into()), "Invalid config: "),
        (RelayError::DecodeError(msg.into()), "Decode error: "),
        (RelayError::TargetingError(msg.into()), "Targeting error: "),
        (RelayError::IoError(msg.into()), "IO error: "),
    ];
    for (err, _) in &list {
        match err {
            RelayError::InvalidConfig(_)
            | RelayError::DecodeError(_)
            | RelayError::TargetingError(_)
            | RelayError::IoError(_) => {}
        }
    }
    list
}
