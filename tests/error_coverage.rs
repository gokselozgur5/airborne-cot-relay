use airborne_cot_relay::RelayError;

/// One value per variant; the exhaustive match fails to compile on a new one.
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

#[test]
fn display_uses_variant_prefix() {
    for (err, prefix) in all_variants("detail") {
        assert_eq!(err.to_string(), format!("{prefix}detail"));
    }
}

#[test]
fn message_is_non_empty_even_without_detail() {
    for (err, _) in all_variants("") {
        assert!(!err.to_string().trim().is_empty(), "{err:?}");
    }
}

#[test]
fn every_variant_is_a_std_error_without_source() {
    for (err, prefix) in all_variants("x") {
        let boxed: Box<dyn std::error::Error> = Box::new(err);
        assert!(boxed.source().is_none());
        assert!(boxed.to_string().starts_with(prefix));
    }
}

#[test]
fn equality_depends_on_variant_and_payload() {
    let a = all_variants("a");
    let b = all_variants("b");
    for (i, (left, _)) in a.iter().enumerate() {
        assert_eq!(left, &all_variants("a")[i].0);
        assert_ne!(left, &b[i].0);
        for (j, (right, _)) in a.iter().enumerate() {
            assert_eq!(i == j, left == right, "{left:?} vs {right:?}");
        }
    }
}
