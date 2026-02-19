//! Middleware system for request/response processing.
//!
//! Provides the [`Middleware`] trait for implementing pre/post request
//! processing hooks. Built-in middleware includes logging, CORS,
//! compression, rate limiting, and static file serving.
//!
//! # Middleware Execution Order
//! 1. `before()` hooks run in registration order
//! 2. Route handler executes
//! 3. `after()` hooks run in **reverse** registration order
//!
//! If any `before()` hook returns `false`, the chain is short-circuited
//! and the response is sent immediately.

pub mod compression;
pub mod cors;
pub mod logger;
pub mod rate_limiter;
pub mod static_files;

use crate::request::Request;
use crate::response::Response;

/// Trait for implementing middleware.
///
/// # Example
/// ```
/// use mini_http::middleware::Middleware;
/// use mini_http::request::Request;
/// use mini_http::response::Response;
///
/// struct AuthMiddleware;
///
/// impl Middleware for AuthMiddleware {
///     fn before(&self, req: &mut Request, res: &mut Response) -> bool {
///         if req.header("authorization").is_none() {
///             res.status(401).send("Unauthorized");
///             return false; // Stop the chain
///         }
///         true // Continue to next middleware/handler
///     }
/// }
/// ```
pub trait Middleware: Send + Sync {
    /// Called before the route handler.
    ///
    /// Return `true` to continue the middleware chain, or `false` to
    /// short-circuit and send the response immediately.
    fn before(&self, req: &mut Request, res: &mut Response) -> bool;

    /// Called after the route handler completes.
    ///
    /// Default implementation does nothing. Override to post-process
    /// the response (e.g., add headers, transform body, log response).
    fn after(&self, _req: &mut Request, _res: &mut Response) {}

    /// Returns a human-readable name for this middleware (used in logging).
    fn name(&self) -> &str {
        "unnamed"
    }
}

/// Error-handling middleware trait.
///
/// Called when a route handler or middleware produces an error condition.
pub trait ErrorHandler: Send + Sync {
    /// Handle an error and produce an appropriate response.
    fn handle_error(&self, error: &crate::error::Error, req: &Request, res: &mut Response);
}

/// Default error handler that returns JSON error responses.
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &crate::error::Error, _req: &Request, res: &mut Response) {
        let status = error.status_code();
        res.status(status).json(&serde_json::json!({
            "error": {
                "code": status,
                "message": error.to_string()
            }
        }));
    }
}
