//! # Mini HTTP Framework
//!
//! A lightweight HTTP framework built from scratch in Rust, inspired by Express.js.
//!
//! ## Features
//! - Express.js-style routing with path parameters (`:id`, `*wildcard`)
//! - Middleware system (logger, CORS, compression, rate limiting, static files)
//! - JSON, HTML, and file responses
//! - Cookie and session support
//! - WebSocket support
//! - TLS/HTTPS via rustls
//! - Thread pool for concurrent connections
//! - HTTP keep-alive
//! - Graceful shutdown
//! - Template rendering via Tera
//! - Shared application state
//!
//! ## Quick Start
//! ```no_run
//! use mini_http::prelude::*;
//!
//! let mut app = App::new();
//!
//! app.use_middleware(Logger::new());
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

pub mod app;
pub mod context;
pub mod cookie;
pub mod error;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod session;
pub mod template;
pub mod thread_pool;
pub mod tls;
pub mod websocket;

/// Commonly used types, re-exported for convenience.
///
/// ```
/// use mini_http::prelude::*;
/// ```
pub mod prelude {
    pub use crate::app::App;
    pub use crate::context::SharedState;
    pub use crate::cookie::{CookieBuilder, SameSite};
    pub use crate::error::{Error, Result};
    pub use crate::middleware::cors::Cors;
    pub use crate::middleware::logger::Logger;
    pub use crate::middleware::Middleware;
    pub use crate::request::{Body, Method, Request};
    pub use crate::response::Response;
    pub use crate::router::RouteGroup;
    pub use crate::session::SessionStore;
    pub use serde_json::json;
}
