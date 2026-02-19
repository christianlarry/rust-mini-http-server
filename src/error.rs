//! Error types for the mini_http framework.
//!
//! Provides a unified [`Error`] type and [`Result`] alias used throughout
//! the framework for consistent error handling.

use std::fmt;
use std::io;

/// Represents all possible errors in the mini_http framework.
#[derive(Debug)]
pub enum Error {
    /// I/O error from networking or file operations.
    Io(io::Error),
    /// HTTP parsing error.
    Parse(String),
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
    /// Route not found.
    NotFound(String),
    /// Method not allowed for the given route.
    MethodNotAllowed(String),
    /// Request body too large.
    PayloadTooLarge,
    /// Request timeout.
    Timeout,
    /// Internal server error.
    Internal(String),
    /// Bad request.
    BadRequest(String),
    /// TLS configuration error.
    Tls(String),
    /// Template rendering error.
    Template(String),
    /// WebSocket error.
    WebSocket(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::NotFound(path) => write!(f, "Not found: {}", path),
            Error::MethodNotAllowed(method) => write!(f, "Method not allowed: {}", method),
            Error::PayloadTooLarge => write!(f, "Payload too large"),
            Error::Timeout => write!(f, "Request timeout"),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
            Error::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Error::Tls(msg) => write!(f, "TLS error: {}", msg),
            Error::Template(msg) => write!(f, "Template error: {}", msg),
            Error::WebSocket(msg) => write!(f, "WebSocket error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<httparse::Error> for Error {
    fn from(e: httparse::Error) -> Self {
        Error::Parse(e.to_string())
    }
}

impl Error {
    /// Returns the appropriate HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            Error::NotFound(_) => 404,
            Error::MethodNotAllowed(_) => 405,
            Error::BadRequest(_) | Error::Parse(_) => 400,
            Error::PayloadTooLarge => 413,
            Error::Timeout => 408,
            Error::Json(_) => 422,
            _ => 500,
        }
    }

    /// Returns the HTTP status text for this error.
    pub fn status_text(&self) -> &str {
        match self {
            Error::NotFound(_) => "Not Found",
            Error::MethodNotAllowed(_) => "Method Not Allowed",
            Error::BadRequest(_) | Error::Parse(_) => "Bad Request",
            Error::PayloadTooLarge => "Payload Too Large",
            Error::Timeout => "Request Timeout",
            Error::Json(_) => "Unprocessable Entity",
            _ => "Internal Server Error",
        }
    }
}

/// Result type alias for mini_http operations.
pub type Result<T> = std::result::Result<T, Error>;
