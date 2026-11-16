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

fn extract_xml_tag<'a>(xml: &'a str, tag_name: &str) -> Option<&'a str> {
    let start_pat = format!("<{tag_name}");
    let start = xml.find(&start_pat)?;
    let rest = &xml[start..];
    let end = rest.find('>')?;
    Some(&rest[..=end])
}

struct EventTagAttrs {
    uid: String,
    cot_type: String,
    how: String,
    time: String,
    start: String,
    stale: String,
}

fn parse_event_tag(event_tag: &str) -> Result<EventTagAttrs, JsonCotError> {
    let uid = extract_xml_attr(event_tag, "uid")
        .ok_or(JsonCotError::MissingField("uid"))?.to_string();
    let cot_type = extract_xml_attr(event_tag, "type")
        .ok_or(JsonCotError::MissingField("type"))?.to_string();
    let how = extract_xml_attr(event_tag, "how").unwrap_or("m-g").to_string();
    let time = extract_xml_attr(event_tag, "time")
        .ok_or(JsonCotError::MissingField("time"))?.to_string();
    let start = extract_xml_attr(event_tag, "start").unwrap_or(&time).to_string();
    let stale = extract_xml_attr(event_tag, "stale").unwrap_or(&time).to_string();
    Ok(EventTagAttrs { uid, cot_type, how, time, start, stale })
}

struct PointTagAttrs {
    lat: f64,
    lon: f64,
    hae: f64,
    ce: f64,
    le: f64,
}

fn parse_point_tag(point_tag: &str) -> Result<PointTagAttrs, JsonCotError> {
    let lat = extract_xml_attr(point_tag, "lat")
        .ok_or(JsonCotError::MissingField("lat"))?
        .parse::<f64>().map_err(|_| JsonCotError::NumericParseError("lat"))?;
    let lon = extract_xml_attr(point_tag, "lon")
        .ok_or(JsonCotError::MissingField("lon"))?
        .parse::<f64>().map_err(|_| JsonCotError::NumericParseError("lon"))?;
    let hae = extract_xml_attr(point_tag, "hae")
        .and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let ce = extract_xml_attr(point_tag, "ce")
        .and_then(|v| v.parse().ok()).unwrap_or(10.0);
    let le = extract_xml_attr(point_tag, "le")
        .and_then(|v| v.parse().ok()).unwrap_or(10.0);
    Ok(PointTagAttrs { lat, lon, hae, ce, le })
}

impl CotEventDto {
    /// Deserializes a Cursor-on-Target XML string into a `CotEventDto`.
    pub fn from_xml(xml: &str) -> Result<Self, JsonCotError> {
        let ev = parse_event_tag(extract_xml_tag(xml, "event")
            .ok_or(JsonCotError::MalformedXml("missing event tag"))?)?;
        let pt = parse_point_tag(extract_xml_tag(xml, "point")
            .ok_or(JsonCotError::MalformedXml("missing point tag"))?)?;
        let track_tag = extract_xml_tag(xml, "track");
        let course = track_tag.and_then(|t| extract_xml_attr(t, "course")).and_then(|v| v.parse().ok());
        let speed = track_tag.and_then(|t| extract_xml_attr(t, "speed")).and_then(|v| v.parse().ok());
        let callsign = extract_xml_tag(xml, "contact")
            .and_then(|t| extract_xml_attr(t, "callsign")).map(ToString::to_string);
        Ok(Self {
            uid: ev.uid, cot_type: ev.cot_type, how: ev.how, time: ev.time,
            start: ev.start, stale: ev.stale, lat: pt.lat, lon: pt.lon, hae: pt.hae,
            ce: pt.ce, le: pt.le, course, speed, callsign,
        })
    }
}

impl CotEventDto {
    /// Serializes this `CotEventDto` into a Cursor-on-Target XML string.
    pub fn to_xml(&self) -> String {
        let mut detail = String::new();
        if let (Some(c), Some(s)) = (self.course, self.speed) {
            detail.push_str(&format!(r#"<track course="{c:.2}" speed="{s:.2}"/>"#));
        }
        if let Some(cs) = &self.callsign {
            detail.push_str(&format!(r#"<contact callsign="{cs}"/>"#));
        }
        format!(
            r#"<event version="2.0" uid="{}" type="{}" how="{}" time="{}" start="{}" stale="{}"><point lat="{:.7}" lon="{:.7}" hae="{:.2}" ce="{:.2}" le="{:.2}"/><detail>{detail}</detail></event>"#,
            self.uid, self.cot_type, self.how, self.time, self.start, self.stale,
            self.lat, self.lon, self.hae, self.ce, self.le
        )
    }
}
