use crate::config::RelayError;
use sightline_cot::cot::{escape_xml_attr, format_iso8601_utc};
use sightline_cot::tracker::Track;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
