//! Bidirectional JSON to Cursor-on-Target (CoT) XML converter.

/// Errors occurring during JSON or XML parsing and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonCotError {
    /// Required field was missing from payload.
    MissingField(&'static str),
    /// Malformed JSON payload.
    MalformedJson(&'static str),
    /// Malformed XML payload.
    MalformedXml(&'static str),
    /// Failed to parse numeric value.
    NumericParseError(&'static str),
}

impl std::fmt::Display for JsonCotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::MalformedJson(msg) => write!(f, "malformed json: {msg}"),
            Self::MalformedXml(msg) => write!(f, "malformed xml: {msg}"),
            Self::NumericParseError(field) => write!(f, "invalid numeric value for: {field}"),
        }
    }
}

impl std::error::Error for JsonCotError {}

/// Canonical intermediate representation of a Cursor-on-Target event.
#[derive(Debug, Clone, PartialEq)]
pub struct CotEventDto {
    pub uid: String,
    pub cot_type: String,
    pub how: String,
    pub time: String,
    pub start: String,
    pub stale: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub ce: f64,
    pub le: f64,
    pub course: Option<f64>,
    pub speed: Option<f64>,
    pub callsign: Option<String>,
}

impl CotEventDto {
    /// Constructs a new `CotEventDto` with mandatory coordinates and event headers.
    pub fn new(
        uid: impl Into<String>,
        cot_type: impl Into<String>,
        time: impl Into<String>,
        lat: f64,
        lon: f64,
    ) -> Self {
        let t = time.into();
        Self {
            uid: uid.into(),
            cot_type: cot_type.into(),
            how: "m-g".to_string(),
            time: t.clone(),
            start: t.clone(),
            stale: t,
            lat,
            lon,
            hae: 0.0,
            ce: 10.0,
            le: 10.0,
            course: None,
            speed: None,
            callsign: None,
        }
    }
}

fn extract_xml_attr<'a>(tag_content: &'a str, attr_name: &str) -> Option<&'a str> {
    let pattern = format!("{attr_name}=\"");
    let start = tag_content.find(&pattern)? + pattern.len();
    let rest = &tag_content[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}
