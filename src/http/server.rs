use super::{parse_request_line, HttpResponse, Method, RequestLine};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

pub type RouteHandler = Box<dyn Fn(&RequestLine) -> HttpResponse + Send + Sync>;

#[derive(Default)]
pub struct Router {
    routes: HashMap<String, RouteHandler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route<F>(&mut self, path: impl Into<String>, handler: F)
    where
        F: Fn(&RequestLine) -> HttpResponse + Send + Sync + 'static,
    {
        self.routes.insert(path.into(), Box::new(handler));
    }
}

impl Router {
    pub fn dispatch(&self, req: &RequestLine) -> HttpResponse {
        if req.method != Method::Get {
            return HttpResponse::method_not_allowed();
        }
        match self.routes.get(&req.path) {
            Some(handler) => handler(req),
            None => HttpResponse::not_found(),
        }
    }
}

pub struct HttpServer {
    listener: TcpListener,
    router: Router,
}

impl HttpServer {
    pub fn bind<A: ToSocketAddrs>(addr: A, router: Router) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener, router })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

impl HttpServer {
    fn handle_connection(&self, mut stream: TcpStream) -> io::Result<()> {
        stream.set_nonblocking(true)?;
        let mut buf = [0u8; 8192];
        let n = match stream.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        };
        let response = match std::str::from_utf8(&buf[..n]) {
            Ok(text) => match parse_request_line(text.lines().next().unwrap_or("")) {
                Ok(req) => self.router.dispatch(&req),
                Err(_) => HttpResponse::bad_request("Bad Request\n"),
            },
            Err(_) => HttpResponse::bad_request("Bad Request\n"),
        };
        let _ = stream.write_all(&response.to_bytes());
        let _ = stream.flush();
        Ok(())
    }
}

impl HttpServer {
    pub fn poll_step(&mut self) -> io::Result<bool> {
        match self.listener.accept() {
            Ok((stream, _)) => {
                self.handle_connection(stream)?;
                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(false),
            Err(e) => Err(e),
        }
    }
}
