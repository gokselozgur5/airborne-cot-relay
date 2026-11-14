use super::{parse_request_line, HttpResponse, Method, RequestLine};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

pub type RouteHandler = Box<dyn Fn(&RequestLine) -> HttpResponse + Send + Sync>;
