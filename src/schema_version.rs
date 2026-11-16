//! CoT XML schema version negotiation and downgrade transformer.

use std::fmt;

/// Errors arising during schema version detection, negotiation, or transformation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaVersionError {
    /// Schema version string is unsupported or invalid.
    UnsupportedVersion(String),
    /// Missing or malformed root event element in XML payload.
    MalformedXml(&'static str),
    /// Schema version negotiation between client and server failed.
    NegotiationFailed,
}

impl fmt::Display for SchemaVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported schema version: {v}"),
            Self::MalformedXml(msg) => write!(f, "malformed xml: {msg}"),
            Self::NegotiationFailed => write!(f, "schema version negotiation failed"),
        }
    }
}

impl std::error::Error for SchemaVersionError {}

