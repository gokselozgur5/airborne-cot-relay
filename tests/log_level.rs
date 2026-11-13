use airborne_cot_relay::log::{Level, ParseLevelError};
use std::str::FromStr;

#[test]
fn test_level_ordering() {
    assert!(Level::Error < Level::Warn);
    assert!(Level::Warn < Level::Info);
    assert!(Level::Info < Level::Debug);
    assert!(Level::Debug < Level::Trace);
}

#[test]
fn test_level_display_and_as_str() {
    assert_eq!(Level::Error.as_str(), "error");
    assert_eq!(Level::Warn.as_str(), "warn");
    assert_eq!(Level::Info.as_str(), "info");
    assert_eq!(Level::Debug.as_str(), "debug");
    assert_eq!(Level::Trace.as_str(), "trace");
    assert_eq!(format!("{}", Level::Info), "info");
}

#[test]
fn test_level_from_str_case_insensitive() {
    assert_eq!(Level::from_str("error").unwrap(), Level::Error);
    assert_eq!(Level::from_str("ERROR").unwrap(), Level::Error);
    assert_eq!("Warn".parse::<Level>().unwrap(), Level::Warn);
    assert_eq!("warning".parse::<Level>().unwrap(), Level::Warn);
    assert_eq!("INFO".parse::<Level>().unwrap(), Level::Info);
    assert_eq!("Debug".parse::<Level>().unwrap(), Level::Debug);
    assert_eq!("TRACE".parse::<Level>().unwrap(), Level::Trace);
}

#[test]
fn test_level_from_str_invalid() {
    let err = "unknown".parse::<Level>().unwrap_err();
    assert_eq!(err, ParseLevelError("unknown".to_string()));
    assert!(format!("{}", err).contains("invalid log level"));
}
