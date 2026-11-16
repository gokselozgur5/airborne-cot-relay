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

/// Recognized Cursor-on-Target XML schema versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CotSchemaVersion {
    /// Legacy schema version 1.0 (or unversioned legacy events).
    V1_0,
    /// Standard modern schema version 2.0.
    #[default]
    V2_0,
}

impl fmt::Display for CotSchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl CotSchemaVersion {
    /// Canonical version string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V2_0 => "2.0",
        }
    }
}

impl CotSchemaVersion {
    /// Parses schema version from version string (e.g. "1.0", "2.0").
    pub fn from_version_str(s: &str) -> Result<Self, SchemaVersionError> {
        match s.trim() {
            "1.0" | "1" => Ok(Self::V1_0),
            "2.0" | "2" => Ok(Self::V2_0),
            other => Err(SchemaVersionError::UnsupportedVersion(other.to_string())),
        }
    }
}

impl CotSchemaVersion {
    /// Detects schema version by inspecting the `<event` tag in CoT XML.
    pub fn detect_from_xml(xml: &str) -> Result<Self, SchemaVersionError> {
        let s = xml
            .find("<event")
            .ok_or(SchemaVersionError::MalformedXml("missing event tag"))?;
        let rest = &xml[s..];
        let e = rest
            .find('>')
            .ok_or(SchemaVersionError::MalformedXml("unclosed event tag"))?;
        let tag = &rest[..e];
        let pattern = "version=\"";
        if let Some(pos) = tag.find(pattern) {
            let start = pos + pattern.len();
            let r = &tag[start..];
            let end = r
                .find('"')
                .ok_or(SchemaVersionError::MalformedXml("unclosed version attr"))?;
            Self::from_version_str(&r[..end])
        } else {
            Ok(Self::V1_0)
        }
    }
}

