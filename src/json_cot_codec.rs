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
    /// Payload exceeded maximum allowable buffer size.
    PayloadTooLarge(usize),
}

impl std::fmt::Display for JsonCotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::MalformedJson(msg) => write!(f, "malformed json: {msg}"),
            Self::MalformedXml(msg) => write!(f, "malformed xml: {msg}"),
            Self::NumericParseError(field) => write!(f, "invalid numeric value for: {field}"),
            Self::PayloadTooLarge(size) => {
                write!(f, "payload exceeded maximum buffer size: {size}")
            }
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

pub(crate) fn find_xml_attr_span(s: &str, name: &str) -> Option<(usize, usize, usize)> {
    let (b, nb) = (s.as_bytes(), name.as_bytes());
    (0..b.len().saturating_sub(nb.len())).find_map(|i| {
        let ok = (i == 0 || b[i - 1].is_ascii_whitespace() || b[i - 1] == b'<' || b[i - 1] == b'?')
            && b[i..].starts_with(nb);
        let mut j = i + nb.len();
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if ok && j < b.len() && b[j] == b'=' {
            j += 1;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                let end = s[j + 1..].find(b[j] as char)?;
                return Some((i, j + 1, j + 1 + end));
            }
        }
        None
    })
}

pub(crate) fn extract_xml_attr<'a>(tag_content: &'a str, attr_name: &str) -> Option<&'a str> {
    find_xml_attr_span(tag_content, attr_name).map(|(_, vs, ve)| &tag_content[vs..ve])
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
    let unesc = crate::xml_sanitize::unescape_xml;
    let uid = extract_xml_attr(event_tag, "uid")
        .map(unesc)
        .ok_or(JsonCotError::MissingField("uid"))?;
    let cot_type = extract_xml_attr(event_tag, "type")
        .map(unesc)
        .ok_or(JsonCotError::MissingField("type"))?;
    let how = extract_xml_attr(event_tag, "how").map_or_else(|| "m-g".to_string(), unesc);
    let time = extract_xml_attr(event_tag, "time")
        .map(unesc)
        .ok_or(JsonCotError::MissingField("time"))?;
    let start = extract_xml_attr(event_tag, "start")
        .map(unesc)
        .unwrap_or_else(|| time.clone());
    let stale = extract_xml_attr(event_tag, "stale")
        .map(unesc)
        .unwrap_or_else(|| time.clone());
    Ok(EventTagAttrs {
        uid,
        cot_type,
        how,
        time,
        start,
        stale,
    })
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
        .parse::<f64>()
        .map_err(|_| JsonCotError::NumericParseError("lat"))?;
    let lon = extract_xml_attr(point_tag, "lon")
        .ok_or(JsonCotError::MissingField("lon"))?
        .parse::<f64>()
        .map_err(|_| JsonCotError::NumericParseError("lon"))?;
    let hae = extract_xml_attr(point_tag, "hae")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let ce = extract_xml_attr(point_tag, "ce")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10.0);
    let le = extract_xml_attr(point_tag, "le")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10.0);
    Ok(PointTagAttrs {
        lat,
        lon,
        hae,
        ce,
        le,
    })
}

impl CotEventDto {
    /// Deserializes a Cursor-on-Target XML string into a `CotEventDto`.
    pub fn from_xml(xml: &str) -> Result<Self, JsonCotError> {
        let ev = parse_event_tag(
            extract_xml_tag(xml, "event").ok_or(JsonCotError::MalformedXml("missing event tag"))?,
        )?;
        let pt = parse_point_tag(
            extract_xml_tag(xml, "point").ok_or(JsonCotError::MalformedXml("missing point tag"))?,
        )?;
        let track_tag = extract_xml_tag(xml, "track");
        let course = track_tag
            .and_then(|t| extract_xml_attr(t, "course"))
            .and_then(|v| v.parse().ok());
        let speed = track_tag
            .and_then(|t| extract_xml_attr(t, "speed"))
            .and_then(|v| v.parse().ok());
        let callsign = extract_xml_tag(xml, "contact")
            .and_then(|t| extract_xml_attr(t, "callsign"))
            .map(crate::xml_sanitize::unescape_xml);
        Ok(Self {
            uid: ev.uid,
            cot_type: ev.cot_type,
            how: ev.how,
            time: ev.time,
            start: ev.start,
            stale: ev.stale,
            lat: pt.lat,
            lon: pt.lon,
            hae: pt.hae,
            ce: pt.ce,
            le: pt.le,
            course,
            speed,
            callsign,
        })
    }
}

impl CotEventDto {
    /// Serializes this `CotEventDto` into a Cursor-on-Target XML string.
    pub fn to_xml(&self) -> String {
        let esc = crate::xml_sanitize::sanitize_and_escape_attr;
        let mut detail = String::new();
        if let (Some(c), Some(s)) = (self.course, self.speed) {
            detail.push_str(&format!(r#"<track course="{c:.2}" speed="{s:.2}"/>"#));
        }
        if let Some(cs) = &self.callsign {
            detail.push_str(&format!(r#"<contact callsign="{}"/>"#, esc(cs)));
        }
        format!(
            r#"<event version="2.0" uid="{}" type="{}" how="{}" time="{}" start="{}" stale="{}"><point lat="{:.7}" lon="{:.7}" hae="{:.2}" ce="{:.2}" le="{:.2}"/><detail>{detail}</detail></event>"#,
            esc(&self.uid),
            esc(&self.cot_type),
            esc(&self.how),
            esc(&self.time),
            esc(&self.start),
            esc(&self.stale),
            self.lat,
            self.lon,
            self.hae,
            self.ce,
            self.le
        )
    }
}

/// Escapes a string for use inside a JSON string literal (RFC 8259 section 7).
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let key_pat = format!("\"{key}\"");
    let key_idx = json.find(&key_pat)?;
    let rest = &json[key_idx + key_pat.len()..];
    let colon_idx = rest.find(':')?;
    let val_part = rest[colon_idx + 1..].trim_start();
    let mut chars = val_part.strip_prefix('"')?.chars();
    let mut out = String::new();
    loop {
        match chars.next()? {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let cp = u32::from_str_radix(&hex, 16).ok()?;
                    out.push(char::from_u32(cp)?);
                }
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
}

fn extract_json_f64_field(json: &str, key: &'static str) -> Result<Option<f64>, JsonCotError> {
    let key_pat = format!("\"{key}\"");
    let Some(i) = json.find(&key_pat) else {
        return Ok(None);
    };
    let rest = &json[i + key_pat.len()..];
    let Some(ci) = rest.find(':') else {
        return Ok(None);
    };
    let val = rest[ci + 1..].trim_start();
    let end = val.find(|c| c == ',' || c == '}').unwrap_or(val.len());
    let v: f64 = val[..end]
        .trim()
        .parse()
        .map_err(|_| JsonCotError::NumericParseError(key))?;
    v.is_finite()
        .then_some(v)
        .ok_or(JsonCotError::NumericParseError(key))
        .map(Some)
}

fn extract_coord(json: &str, key: &'static str, min: f64, max: f64) -> Result<f64, JsonCotError> {
    let v = extract_json_f64_field(json, key)?.ok_or(JsonCotError::MissingField(key))?;
    (min..=max)
        .contains(&v)
        .then_some(v)
        .ok_or(JsonCotError::NumericParseError(key))
}

impl CotEventDto {
    /// Deserializes a JSON string into a `CotEventDto`.
    pub fn from_json(json: &str) -> Result<Self, JsonCotError> {
        let trimmed = json.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err(JsonCotError::MalformedJson("expected root JSON object"));
        }
        let uid = extract_json_string(json, "uid").ok_or(JsonCotError::MissingField("uid"))?;
        let cot_type =
            extract_json_string(json, "type").ok_or(JsonCotError::MissingField("type"))?;
        let how = extract_json_string(json, "how").unwrap_or_else(|| "m-g".to_string());
        let time = extract_json_string(json, "time").ok_or(JsonCotError::MissingField("time"))?;
        let start = extract_json_string(json, "start").unwrap_or_else(|| time.clone());
        let stale = extract_json_string(json, "stale").unwrap_or_else(|| time.clone());
        let lat = extract_coord(json, "lat", -90.0, 90.0)?;
        let lon = extract_coord(json, "lon", -180.0, 180.0)?;
        let hae = extract_json_f64_field(json, "hae")?.unwrap_or(0.0);
        let ce = extract_json_f64_field(json, "ce")?.unwrap_or(10.0);
        let le = extract_json_f64_field(json, "le")?.unwrap_or(10.0);
        let course = extract_json_f64_field(json, "course")?;
        let speed = extract_json_f64_field(json, "speed")?;
        let callsign = extract_json_string(json, "callsign");
        Ok(Self {
            uid,
            cot_type,
            how,
            time,
            start,
            stale,
            lat,
            lon,
            hae,
            ce,
            le,
            course,
            speed,
            callsign,
        })
    }
}

impl CotEventDto {
    /// Serializes this `CotEventDto` into a JSON string.
    pub fn to_json(&self) -> String {
        let mut fields = format!(
            r#""uid":"{}","type":"{}","how":"{}","time":"{}","start":"{}","stale":"{}","lat":{:.7},"lon":{:.7},"hae":{:.2},"ce":{:.2},"le":{:.2}"#,
            escape_json(&self.uid),
            escape_json(&self.cot_type),
            escape_json(&self.how),
            escape_json(&self.time),
            escape_json(&self.start),
            escape_json(&self.stale),
            self.lat,
            self.lon,
            self.hae,
            self.ce,
            self.le
        );
        if let Some(c) = self.course {
            fields.push_str(&format!(r#","course":{c:.2}"#));
        }
        if let Some(s) = self.speed {
            fields.push_str(&format!(r#","speed":{s:.2}"#));
        }
        if let Some(cs) = &self.callsign {
            fields.push_str(&format!(r#","callsign":"{}""#, escape_json(cs)));
        }
        format!("{{{fields}}}")
    }
}

/// Converts a JSON string directly into a Cursor-on-Target XML string.
pub fn json_to_cot_xml(json: &str) -> Result<String, JsonCotError> {
    CotEventDto::from_json(json).map(|dto| dto.to_xml())
}

/// Converts a Cursor-on-Target XML string directly into a JSON string.
pub fn cot_xml_to_json(xml: &str) -> Result<String, JsonCotError> {
    CotEventDto::from_xml(xml).map(|dto| dto.to_json())
}

/// Default maximum buffer size for streaming decoders (16 MiB).
pub const DEFAULT_MAX_STREAM_BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Streaming converter from newline-delimited JSON into Cursor-on-Target XML events.
#[derive(Debug)]
pub struct JsonToCotStream {
    buf: String,
    max_buf: usize,
}

impl Default for JsonToCotStream {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonToCotStream {
    /// Constructs a new empty `JsonToCotStream`.
    pub fn new() -> Self {
        Self::with_max_buffer_size(DEFAULT_MAX_STREAM_BUFFER_SIZE)
    }

    /// Constructs a new empty `JsonToCotStream` with a custom buffer limit.
    pub fn with_max_buffer_size(max_buf: usize) -> Self {
        Self {
            buf: String::new(),
            max_buf,
        }
    }

    /// Feeds incremental chunk of text/bytes into the stream buffer.
    pub fn feed(&mut self, chunk: &str) {
        self.buf.push_str(chunk);
    }

    /// Decodes the next available CoT XML event if a complete JSON line is buffered.
    pub fn decode_next(&mut self) -> Result<Option<String>, JsonCotError> {
        if let Some(pos) = self.buf.find('\n') {
            let line = self.buf[..pos].trim().to_string();
            self.buf.drain(..=pos);
            if line.is_empty() {
                return Ok(None);
            }
            return json_to_cot_xml(&line).map(Some);
        }
        if self.buf.len() > self.max_buf {
            let len = self.buf.len();
            self.buf.clear();
            return Err(JsonCotError::PayloadTooLarge(len));
        }
        Ok(None)
    }
}

/// Streaming converter from Cursor-on-Target XML events into JSON strings.
#[derive(Debug)]
pub struct CotToJsonStream {
    buf: String,
    max_buf: usize,
}

impl Default for CotToJsonStream {
    fn default() -> Self {
        Self::new()
    }
}

impl CotToJsonStream {
    /// Constructs a new empty `CotToJsonStream`.
    pub fn new() -> Self {
        Self::with_max_buffer_size(DEFAULT_MAX_STREAM_BUFFER_SIZE)
    }

    /// Constructs a new empty `CotToJsonStream` with a custom buffer limit.
    pub fn with_max_buffer_size(max_buf: usize) -> Self {
        Self {
            buf: String::new(),
            max_buf,
        }
    }

    /// Feeds incremental chunk of text/bytes into the stream buffer.
    pub fn feed(&mut self, chunk: &str) {
        self.buf.push_str(chunk);
    }

    /// Decodes the next available JSON object if a complete CoT XML event is buffered.
    pub fn decode_next(&mut self) -> Result<Option<String>, JsonCotError> {
        let end_tag = "</event>";
        if let Some(pos) = self.buf.find(end_tag) {
            let total = pos + end_tag.len();
            let xml = self.buf[..total].to_string();
            self.buf.drain(..total);
            return cot_xml_to_json(&xml).map(Some);
        }
        if self.buf.len() > self.max_buf {
            let len = self.buf.len();
            self.buf.clear();
            return Err(JsonCotError::PayloadTooLarge(len));
        }
        Ok(None)
    }
}
