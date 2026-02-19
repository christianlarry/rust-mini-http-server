//! CORS (Cross-Origin Resource Sharing) middleware.
//!
//! Automatically handles CORS preflight `OPTIONS` requests and adds
//! appropriate CORS headers to all responses.

use crate::middleware::Middleware;
use crate::request::{Method, Request};
use crate::response::Response;

/// CORS middleware with configurable allowed origins, methods, and headers.
///
/// # Example
/// ```
/// use mini_http::middleware::cors::Cors;
///
/// // Allow all origins
/// let cors = Cors::permissive();
///
/// // Or configure specific origins
/// let cors = Cors::new()
///     .allow_origin("https://example.com")
///     .allow_methods("GET, POST, PUT, DELETE")
///     .allow_headers("Content-Type, Authorization")
///     .max_age(3600);
/// ```
pub struct Cors {
    allow_origin: String,
    allow_methods: String,
    allow_headers: String,
    allow_credentials: bool,
    expose_headers: String,
    max_age: Option<u64>,
}

impl Cors {
    /// Create a new CORS middleware with restrictive defaults.
    pub fn new() -> Self {
        Cors {
            allow_origin: String::new(),
            allow_methods: "GET, HEAD, OPTIONS".to_string(),
            allow_headers: "Content-Type".to_string(),
            allow_credentials: false,
            expose_headers: String::new(),
            max_age: None,
        }
    }

    /// Create a permissive CORS middleware that allows all origins and methods.
    pub fn permissive() -> Self {
        Cors {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization, Accept, Origin, X-Requested-With"
                .to_string(),
            allow_credentials: false,
            expose_headers: String::new(),
            max_age: Some(86400),
        }
    }

    /// Set the allowed origin (use `*` for all origins).
    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = origin.to_string();
        self
    }

    /// Set the allowed HTTP methods.
    pub fn allow_methods(mut self, methods: &str) -> Self {
        self.allow_methods = methods.to_string();
        self
    }

    /// Set the allowed request headers.
    pub fn allow_headers(mut self, headers: &str) -> Self {
        self.allow_headers = headers.to_string();
        self
    }

    /// Enable or disable credentials support.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set headers that the browser is allowed to access.
    pub fn expose_headers(mut self, headers: &str) -> Self {
        self.expose_headers = headers.to_string();
        self
    }

    /// Set the preflight cache duration in seconds.
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }
}

impl Default for Cors {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Cors {
    fn before(&self, req: &mut Request, res: &mut Response) -> bool {
        // Determine the origin to use
        let origin = if self.allow_origin == "*" {
            "*".to_string()
        } else if !self.allow_origin.is_empty() {
            self.allow_origin.clone()
        } else {
            // Reflect the request origin if configured origin is empty
            req.header("origin").unwrap_or("*").to_string()
        };

        res.header("Access-Control-Allow-Origin", &origin);
        res.header("Access-Control-Allow-Methods", &self.allow_methods);
        res.header("Access-Control-Allow-Headers", &self.allow_headers);

        if self.allow_credentials {
            res.header("Access-Control-Allow-Credentials", "true");
        }

        if !self.expose_headers.is_empty() {
            res.header("Access-Control-Expose-Headers", &self.expose_headers);
        }

        if let Some(max_age) = self.max_age {
            res.header("Access-Control-Max-Age", &max_age.to_string());
        }

        // Handle preflight (OPTIONS) requests — respond immediately
        if req.method == Method::OPTIONS {
            res.status(204).send_status(204);
            return false; // Don't continue to handler
        }

        true
    }

    fn name(&self) -> &str {
        "cors"
    }
}
