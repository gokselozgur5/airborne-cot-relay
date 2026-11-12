#![allow(clippy::field_reassign_with_default)]
use airborne_cot_relay::{RelayConfig, RelayError};
use std::collections::HashMap;

fn build(pairs: &[(&str, &str)]) -> Result<RelayConfig, RelayError> {
    let env: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    RelayConfig::default().with_env_overrides(|k| env.get(k).cloned())
}

#[test]
fn empty_env_keeps_defaults() {
    assert_eq!(build(&[]).unwrap(), RelayConfig::default());
}

