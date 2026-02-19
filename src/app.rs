//! Application facade — the main entry point for building an HTTP application.
//!
//! `App` provides a high-level, Express.js-inspired API for registering routes,
//! middleware, configuring the server, and starting the HTTP listener.
//!
//! # Example
//! ```no_run
//! use mini_http::prelude::*;
//!
//! let mut app = App::new();
//!
//! app.get("/", |_req, res| {
//!     res.send("Hello, World!");
//! });
//!
//! app.get("/users/:id", |req, res| {
//!     let id = req.param("id").unwrap_or("unknown");
//!     res.json(&serde_json::json!({"user_id": id}));
//! });
//!
//! app.listen("127.0.0.1:8080");
//! ```

use std::sync::Arc;
use std::time::Duration;

use crate::context::{new_shared_state, SharedState};
use crate::middleware::Middleware;
use crate::request::{Method, Request};
use crate::response::Response;
use crate::router::{RouteGroup, Router};
use crate::server::{Server, ServerConfig};
use crate::tls::TlsConfig;
use crate::websocket::WebSocket;

/// The main application builder and HTTP server facade.
pub struct App {
    router: Router,
    middleware: Vec<Box<dyn Middleware>>,
    config: ServerConfig,
    state: Option<SharedState>,
}

impl App {
    /// Create a new application with default settings.
    pub fn new() -> Self {
        App {
            router: Router::new(),
            middleware: Vec::new(),
            config: ServerConfig::default(),
            state: None,
        }
    }

    // ── Route Registration ───────────────────────────────────────────

    /// Register a GET route.
    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::GET, path, handler);
    }

    /// Register a POST route.
    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::POST, path, handler);
    }

    /// Register a PUT route.
    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::PUT, path, handler);
    }

    /// Register a DELETE route.
    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::DELETE, path, handler);
    }

    /// Register a PATCH route.
    pub fn patch<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::PATCH, path, handler);
    }

    /// Register a HEAD route.
    pub fn head<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::HEAD, path, handler);
    }

    /// Register an OPTIONS route.
    pub fn options<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(Method::OPTIONS, path, handler);
    }

    /// Register a route for any HTTP method.
    pub fn route<F>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.router.add(method, path, handler);
    }

    /// Register a route group with a shared prefix.
    ///
    /// # Example
    /// ```no_run
    /// use mini_http::prelude::*;
    ///
    /// let mut app = App::new();
    /// let mut api = RouteGroup::new("/api/v1");
    /// api.get("/users", |_req, res| res.send("list users"));
    /// api.post("/users", |_req, res| res.status(201).send("created"));
    /// app.group(api);
    /// app.listen("127.0.0.1:8080");
    /// ```
    pub fn group(&mut self, group: RouteGroup) {
        self.router.add_group(group);
    }

    /// Register a WebSocket route.
    ///
    /// # Example
    /// ```no_run
    /// use mini_http::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.websocket("/ws", |mut ws| {
    ///     while let Ok(Some(msg)) = ws.read_message() {
    ///         match msg {
    ///             mini_http::websocket::Message::Text(text) => {
    ///                 let _ = ws.send_text(&text);
    ///             }
    ///             mini_http::websocket::Message::Close => break,
    ///             _ => {}
    ///         }
    ///     }
    /// });
    /// app.listen("127.0.0.1:8080");
    /// ```
    pub fn websocket<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(WebSocket) + Send + Sync + 'static,
    {
        self.router.add_ws(path, handler);
    }

    // ── Middleware ────────────────────────────────────────────────────

    /// Add a middleware to the application.
    ///
    /// Middleware are executed in the order they are registered.
    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middleware.push(Box::new(middleware));
    }

    /// Serve static files from a directory at a URL prefix.
    ///
    /// Shorthand for adding a `StaticFiles` middleware.
    ///
    /// # Example
    /// ```no_run
    /// use mini_http::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.serve_static("/public", "./public");
    /// app.listen("127.0.0.1:8080");
    /// ```
    pub fn serve_static(&mut self, url_prefix: &str, root_dir: &str) {
        self.middleware.push(Box::new(
            crate::middleware::static_files::StaticFiles::new(url_prefix, root_dir),
        ));
    }

    // ── Configuration ────────────────────────────────────────────────

    /// Set the number of worker threads.
    pub fn threads(mut self, count: usize) -> Self {
        self.config.threads = count;
        self
    }

    /// Set the request read timeout.
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.config.read_timeout = timeout;
        self
    }

    /// Set the response write timeout.
    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.config.write_timeout = timeout;
        self
    }

    /// Set the maximum request body size in bytes.
    pub fn max_request_size(mut self, size: usize) -> Self {
        self.config.max_request_size = size;
        self
    }

    /// Enable or disable HTTP keep-alive.
    pub fn keep_alive(mut self, enabled: bool) -> Self {
        self.config.keep_alive = enabled;
        self
    }

    /// Set the keep-alive timeout.
    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.config.keep_alive_timeout = timeout;
        self
    }

    /// Configure TLS/HTTPS with certificate and key files.
    ///
    /// # Example
    /// ```no_run
    /// use mini_http::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.tls("certs/cert.pem", "certs/key.pem");
    /// app.listen("0.0.0.0:443");
    /// ```
    pub fn tls(&mut self, cert_path: &str, key_path: &str) {
        match TlsConfig::new(cert_path, key_path) {
            Ok(tls) => self.config.tls = Some(tls),
            Err(e) => panic!("TLS configuration error: {}", e),
        }
    }

    /// Enable or disable graceful shutdown.
    pub fn graceful_shutdown(mut self, enabled: bool) -> Self {
        self.config.graceful_shutdown = enabled;
        self
    }

    // ── Shared State ─────────────────────────────────────────────────

    /// Set shared application state.
    ///
    /// State is accessible from handlers via `req.app_state()`.
    ///
    /// # Example
    /// ```no_run
    /// use mini_http::prelude::*;
    /// use mini_http::context::new_shared_state;
    ///
    /// let state = new_shared_state();
    /// {
    ///     let mut s = state.write().unwrap();
    ///     s.insert("app_name", "My App".to_string());
    /// }
    ///
    /// let mut app = App::new();
    /// app.set_state(state);
    /// ```
    pub fn set_state(&mut self, state: SharedState) {
        self.state = Some(state);
    }

    /// Create and set a new shared state, returning a clone for external use.
    pub fn with_state(&mut self) -> SharedState {
        let state = new_shared_state();
        self.state = Some(state.clone());
        state
    }

    // ── Server Start ─────────────────────────────────────────────────

    /// Start the HTTP server and listen for connections.
    ///
    /// This method blocks the current thread. The server will handle
    /// incoming connections using the configured thread pool.
    pub fn listen(self, addr: &str) {
        let router = Arc::new(self.router);
        let middleware = Arc::new(self.middleware);

        Server::start(addr, router, middleware, self.config, self.state);
    }

    /// Alias for `listen` — start the server (Express.js-style naming).
    pub fn run(self, addr: &str) {
        self.listen(addr);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
