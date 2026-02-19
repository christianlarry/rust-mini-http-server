//! TCP server with thread pool, TLS, keep-alive, and graceful shutdown.
//!
//! The server accepts TCP connections, optionally wraps them with TLS,
//! and dispatches them to worker threads via a thread pool. Each connection
//! is handled in a loop for HTTP keep-alive support.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::context::SharedState;
use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;
use crate::router::Router;
use crate::thread_pool::ThreadPool;
use crate::tls::TlsConfig;
use crate::websocket::WebSocket;

/// Server configuration.
pub struct ServerConfig {
    /// Number of worker threads (default: number of CPUs or 4).
    pub threads: usize,
    /// Read timeout per request (default: 30s).
    pub read_timeout: Duration,
    /// Write timeout per response (default: 30s).
    pub write_timeout: Duration,
    /// Maximum request size in bytes (default: 10MB).
    pub max_request_size: usize,
    /// Enable keep-alive (default: true).
    pub keep_alive: bool,
    /// Keep-alive timeout (default: 5s).
    pub keep_alive_timeout: Duration,
    /// Optional TLS configuration.
    pub tls: Option<TlsConfig>,
    /// Enable graceful shutdown on SIGINT/SIGTERM.
    pub graceful_shutdown: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            max_request_size: 10 * 1024 * 1024, // 10MB
            keep_alive: true,
            keep_alive_timeout: Duration::from_secs(5),
            tls: None,
            graceful_shutdown: true,
        }
    }
}

/// TCP/TLS stream abstraction for unified reading/writing.
enum Connection {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ServerConnection, TcpStream>>),
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Connection::Plain(s) => s.read(buf),
            Connection::Tls(s) => s.read(buf),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Connection::Plain(s) => s.write(buf),
            Connection::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Connection::Plain(s) => s.flush(),
            Connection::Tls(s) => s.flush(),
        }
    }
}

impl Connection {
    fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self {
            Connection::Plain(s) => s.peer_addr(),
            Connection::Tls(s) => s.get_ref().peer_addr(),
        }
    }

    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match self {
            Connection::Plain(s) => s.set_read_timeout(timeout),
            Connection::Tls(s) => s.get_ref().set_read_timeout(timeout),
        }
    }

    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        match self {
            Connection::Plain(s) => s.set_write_timeout(timeout),
            Connection::Tls(s) => s.get_ref().set_write_timeout(timeout),
        }
    }
}

/// The HTTP server.
pub struct Server;

impl Server {
    /// Start the server and begin accepting connections.
    pub fn start(
        addr: &str,
        router: Arc<Router>,
        middleware: Arc<Vec<Box<dyn Middleware>>>,
        config: ServerConfig,
        state: Option<SharedState>,
    ) {
        let listener = TcpListener::bind(addr).unwrap_or_else(|e| {
            panic!("Failed to bind to {}: {}", addr, e);
        });

        let protocol = if config.tls.is_some() { "https" } else { "http" };
        eprintln!("🚀 Server running at {}://{}", protocol, addr);
        eprintln!("   Threads: {}", config.threads);
        if config.keep_alive {
            eprintln!("   Keep-Alive: enabled ({}s timeout)", config.keep_alive_timeout.as_secs());
        }

        let pool = ThreadPool::new(config.threads);
        let running = Arc::new(AtomicBool::new(true));
        let config = Arc::new(config);

        // Set up graceful shutdown
        if config.graceful_shutdown {
            let running_clone = Arc::clone(&running);
            let _ = ctrlc::set_handler(move || {
                eprintln!("\n🛑 Shutting down gracefully...");
                running_clone.store(false, Ordering::SeqCst);
            });
        }

        // Set listener to non-blocking for graceful shutdown polling
        listener.set_nonblocking(true).expect("Failed to set non-blocking");

        while running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let router = Arc::clone(&router);
                    let middleware = Arc::clone(&middleware);
                    let config = Arc::clone(&config);
                    let state = state.clone();

                    pool.execute(move || {
                        // Set stream back to blocking mode
                        if stream.set_nonblocking(false).is_err() {
                            return;
                        }

                        // Wrap with TLS if configured
                        let conn = if let Some(ref tls_config) = config.tls {
                            match tls_config.accept(stream) {
                                Ok(tls_stream) => Connection::Tls(Box::new(tls_stream)),
                                Err(e) => {
                                    log::error!("TLS handshake failed: {}", e);
                                    return;
                                }
                            }
                        } else {
                            Connection::Plain(stream)
                        };

                        Self::handle_connection(conn, &router, &middleware, &config, state);
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    log::error!("Connection accept error: {}", e);
                }
            }
        }

        eprintln!("Server stopped.");
    }

    /// Handle a single TCP connection (possibly multiple requests with keep-alive).
    fn handle_connection(
        mut conn: Connection,
        router: &Router,
        middleware: &[Box<dyn Middleware>],
        config: &ServerConfig,
        state: Option<SharedState>,
    ) {
        let remote_addr = conn.peer_addr().ok();

        let _ = conn.set_write_timeout(Some(config.write_timeout));

        // Connection loop for keep-alive
        loop {
            // Set read timeout (shorter for keep-alive subsequent requests)
            let _ = conn.set_read_timeout(Some(config.read_timeout));

            // Read request data
            let buffer = match read_request(&mut conn, config.max_request_size) {
                Ok(buf) if buf.is_empty() => break, // Connection closed
                Ok(buf) => buf,
                Err(_) => break, // Timeout or read error
            };

            // Parse request
            let mut req = match Request::parse(&buffer) {
                Ok(req) => req,
                Err(e) => {
                    let mut res = Response::new();
                    res.status(400).send(&format!("Bad Request: {}", e));
                    let _ = conn.write_all(&res.to_bytes());
                    break;
                }
            };

            req.remote_addr = remote_addr;
            req.state = state.clone();

            // Check for WebSocket upgrade
            if req.is_websocket_upgrade() {
                if let Some(client_key) = req.header("sec-websocket-key").map(String::from) {
                    if let Some((handler, params)) = router.match_ws_route(&req.path) {
                        req.params = params;

                        // Send handshake response
                        let handshake = WebSocket::handshake_response(&client_key);
                        if conn.write_all(&handshake).is_err() {
                            break;
                        }
                        let _ = conn.flush();

                        // Create WebSocket and call handler
                        let ws = WebSocket::new(Box::new(conn));
                        handler(ws);
                        return; // WebSocket handler owns the connection
                    }
                }
                // No WebSocket route found
                let mut res = Response::new();
                res.status(404).send("WebSocket endpoint not found");
                let _ = conn.write_all(&res.to_bytes());
                break;
            }

            // Normal HTTP request handling
            let mut res = Response::new();

            // Run middleware before hooks
            let mut middleware_passed = true;
            for mw in middleware.iter() {
                if !mw.before(&mut req, &mut res) {
                    middleware_passed = false;
                    break;
                }
            }

            // Run route handler (only if all middleware passed)
            if middleware_passed {
                if !router.handle(&mut req, &mut res) {
                    // No route matched — 404
                    res.status(404).json(&serde_json::json!({
                        "error": {
                            "code": 404,
                            "message": format!("Cannot {} {}", req.method, req.path)
                        }
                    }));
                }
            }

            // Run middleware after hooks (reverse order)
            for mw in middleware.iter().rev() {
                mw.after(&mut req, &mut res);
            }

            // Determine keep-alive
            let connection_header = req
                .header("connection")
                .unwrap_or("")
                .to_lowercase();

            let should_keep_alive = config.keep_alive
                && req.http_version.contains("1.1")
                && connection_header != "close";

            // Set connection header on response
            if should_keep_alive {
                res.header("Connection", "keep-alive");
                res.header(
                    "Keep-Alive",
                    &format!("timeout={}", config.keep_alive_timeout.as_secs()),
                );
            } else {
                res.header("Connection", "close");
            }

            // Write response
            let response_bytes = res.to_bytes();
            if conn.write_all(&response_bytes).is_err() {
                break;
            }
            let _ = conn.flush();

            if !should_keep_alive {
                break;
            }

            // Set shorter timeout for next keep-alive request
            let _ = conn.set_read_timeout(Some(config.keep_alive_timeout));
        }
    }
}

/// Read a complete HTTP request from the stream.
///
/// Reads headers first, then reads the body based on Content-Length.
/// Returns an empty Vec if the connection is closed.
fn read_request(stream: &mut impl Read, max_size: usize) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(8192);
    let mut temp = [0u8; 4096];
    let mut headers_complete = false;
    let mut header_end = 0;
    let mut content_length: usize = 0;

    loop {
        let n = match stream.read(&mut temp) {
            Ok(0) => return Ok(Vec::new()), // Connection closed
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                if buffer.is_empty() {
                    return Ok(Vec::new());
                }
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                if buffer.is_empty() {
                    return Ok(Vec::new());
                }
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Request timeout"));
            }
            Err(e) => return Err(e),
        };

        buffer.extend_from_slice(&temp[..n]);

        // Security: check max request size
        if buffer.len() > max_size {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Request too large",
            ));
        }

        if !headers_complete {
            // Search for end of headers: \r\n\r\n
            if let Some(pos) = find_header_end(&buffer) {
                headers_complete = true;
                header_end = pos + 4;

                // Parse Content-Length from headers
                let headers_str = String::from_utf8_lossy(&buffer[..pos]);
                for line in headers_str.lines() {
                    if line.to_lowercase().starts_with("content-length:") {
                        content_length = line[15..].trim().parse().unwrap_or(0);
                    }
                }
            }
        }

        if headers_complete {
            let body_received = buffer.len() - header_end;
            if body_received >= content_length {
                break;
            }
        }
    }

    Ok(buffer)
}

/// Find the position of `\r\n\r\n` in a byte slice.
fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4)
        .position(|w| w == b"\r\n\r\n")
}
