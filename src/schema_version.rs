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

/// Negotiates CoT schema version between client preferences and server limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaNegotiator {
    server_version: CotSchemaVersion,
    min_supported: CotSchemaVersion,
    allow_downgrade: bool,
}

impl SchemaNegotiator {
    /// Creates a negotiator with default modern server version and downgrade enabled.
    pub fn new(server_version: CotSchemaVersion) -> Self {
        Self {
            server_version,
            min_supported: CotSchemaVersion::V1_0,
            allow_downgrade: true,
        }
    }
}

impl SchemaNegotiator {
    /// Sets the minimum supported schema version.
    pub fn with_min_supported(mut self, min: CotSchemaVersion) -> Self {
        self.min_supported = min;
        self
    }

    /// Sets whether downgrade from server version is allowed.
    pub fn with_allow_downgrade(mut self, allow: bool) -> Self {
        self.allow_downgrade = allow;
        self
    }

    /// Returns configured server version.
    pub fn server_version(&self) -> CotSchemaVersion {
        self.server_version
    }
}

impl SchemaNegotiator {
    /// Negotiates an agreed schema version given a client's requested version.
    pub fn negotiate(
        &self,
        requested: CotSchemaVersion,
    ) -> Result<CotSchemaVersion, SchemaVersionError> {
        if requested == self.server_version {
            return Ok(self.server_version);
        }
        if requested < self.server_version {
            if self.allow_downgrade && requested >= self.min_supported {
                Ok(requested)
            } else {
                Err(SchemaVersionError::NegotiationFailed)
            }
        } else if self.server_version >= self.min_supported {
            Ok(self.server_version)
        } else {
            Err(SchemaVersionError::NegotiationFailed)
        }
    }
}

impl SchemaNegotiator {
    /// Negotiates the highest mutually supported version from client capabilities.
    pub fn negotiate_best(
        &self,
        client_caps: &[CotSchemaVersion],
    ) -> Result<CotSchemaVersion, SchemaVersionError> {
        let mut supported: Vec<CotSchemaVersion> = client_caps
            .iter()
            .copied()
            .filter_map(|ver| self.negotiate(ver).ok())
            .collect();
        supported.sort();
        supported.pop().ok_or(SchemaVersionError::NegotiationFailed)
    }

    /// Negotiates schema version from client version header (e.g. "2.0").
    pub fn negotiate_from_header(
        &self,
        header: &str,
    ) -> Result<CotSchemaVersion, SchemaVersionError> {
        let requested = CotSchemaVersion::from_version_str(header)?;
        self.negotiate(requested)
    }
}

/// Transforms CoT XML by downgrading schema version and stripping incompatible elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DowngradeTransformer {
    target_version: CotSchemaVersion,
    strip_flow_tags: bool,
}

impl DowngradeTransformer {
    /// Creates transformer targeting specified version.
    pub fn new(target_version: CotSchemaVersion) -> Self {
        Self {
            target_version,
            strip_flow_tags: true,
        }
    }

    /// Enables or disables stripping of `<_flow-tags_>` elements during downgrade.
    pub fn with_strip_flow_tags(mut self, strip: bool) -> Self {
        self.strip_flow_tags = strip;
        self
    }
}

fn strip_flow_tags_element(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut cursor = 0;
    while let Some(pos) = xml[cursor..].find("<_flow-tags_") {
        let start = cursor + pos;
        out.push_str(&xml[cursor..start]);
        if let Some(end_pos) = xml[start..].find("/>") {
            cursor = start + end_pos + 2;
        } else if let Some(close_pos) = xml[start..].find("</_flow-tags_>") {
            cursor = start + close_pos + "</_flow-tags_>".len();
        } else {
            out.push_str(&xml[start..]);
            cursor = xml.len();
            break;
        }
    }
    out.push_str(&xml[cursor..]);
    out
}

impl DowngradeTransformer {
    /// Transforms XML payload to the target schema version.
    pub fn transform(&self, xml: &str) -> Result<String, SchemaVersionError> {
        let current_ver = CotSchemaVersion::detect_from_xml(xml)?;
        if current_ver <= self.target_version && !self.strip_flow_tags {
            return Ok(xml.to_string());
        }
        let s = xml
            .find("<event")
            .ok_or(SchemaVersionError::MalformedXml("missing event tag"))?;
        let e = xml[s..]
            .find('>')
            .ok_or(SchemaVersionError::MalformedXml("unclosed event tag"))?
            + s;
        let (before, tag, after) = (&xml[..s], &xml[s..=e], &xml[e + 1..]);
        let new_tag = if let Some(vp) = tag.find("version=\"") {
            let vs = vp + "version=\"".len();
            let ve = tag[vs..]
                .find('"')
                .ok_or(SchemaVersionError::MalformedXml("unclosed version attr"))?
                + vs;
            format!(
                "{}{}{}",
                &tag[..vs],
                self.target_version.as_str(),
                &tag[ve..]
            )
        } else {
            format!(
                "<event version=\"{}\"{}",
                self.target_version.as_str(),
                &tag["<event".len()..]
            )
        };
        let mut res = format!("{before}{new_tag}{after}");
        if self.strip_flow_tags {
            res = strip_flow_tags_element(&res);
        }
        Ok(res)
    }
}
