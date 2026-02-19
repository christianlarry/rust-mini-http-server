//! HTTP response builder and representation.
//!
//! Provides the [`Response`] struct with a builder-pattern API for
//! constructing HTTP responses. Supports multiple response types including
//! plain text, JSON, HTML, file serving, and redirects.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::cookie::CookieBuilder;

/// Common HTTP status code constants.
pub mod status {
    pub const OK: u16 = 200;
    pub const CREATED: u16 = 201;
    pub const NO_CONTENT: u16 = 204;
    pub const MOVED_PERMANENTLY: u16 = 301;
    pub const FOUND: u16 = 302;
    pub const NOT_MODIFIED: u16 = 304;
    pub const TEMPORARY_REDIRECT: u16 = 307;
    pub const PERMANENT_REDIRECT: u16 = 308;
    pub const BAD_REQUEST: u16 = 400;
    pub const UNAUTHORIZED: u16 = 401;
    pub const FORBIDDEN: u16 = 403;
    pub const NOT_FOUND: u16 = 404;
    pub const METHOD_NOT_ALLOWED: u16 = 405;
    pub const CONFLICT: u16 = 409;
    pub const PAYLOAD_TOO_LARGE: u16 = 413;
    pub const UNPROCESSABLE_ENTITY: u16 = 422;
    pub const TOO_MANY_REQUESTS: u16 = 429;
    pub const INTERNAL_SERVER_ERROR: u16 = 500;
    pub const BAD_GATEWAY: u16 = 502;
    pub const SERVICE_UNAVAILABLE: u16 = 503;
}

/// Returns the standard status text for an HTTP status code.
pub fn status_text(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}

/// Represents an HTTP response being built.
///
/// Uses a builder pattern to accumulate status, headers, and body
/// before the server writes it to the client connection.
///
/// # Example
/// ```
/// use mini_http::response::Response;
///
/// let mut res = Response::new();
/// res.status(200)
///    .header("X-Custom", "value")
///    .send("Hello, World!");
/// ```
#[derive(Debug)]
pub struct Response {
    /// HTTP status code.
    pub status_code: u16,
    /// Response headers (multiple values per key supported via set_cookie).
    pub headers: HashMap<String, String>,
    /// Set-Cookie headers (stored separately to allow multiple).
    pub set_cookies: Vec<String>,
    /// Response body bytes.
    pub body: Vec<u8>,
    /// Whether the response has been finalized.
    pub sent: bool,
    /// Whether this is a WebSocket upgrade response.
    pub is_upgrade: bool,
}

impl Response {
    /// Create a new response with 200 OK status.
    pub fn new() -> Self {
        Response {
            status_code: 200,
            headers: HashMap::new(),
            set_cookies: Vec::new(),
            body: Vec::new(),
            sent: false,
            is_upgrade: false,
        }
    }

    /// Set the HTTP status code. Returns `&mut Self` for chaining.
    pub fn status(&mut self, code: u16) -> &mut Self {
        self.status_code = code;
        self
    }

    /// Set a response header. Returns `&mut Self` for chaining.
    pub fn header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Send a plain text response body.
    pub fn send(&mut self, body: &str) {
        if !self.headers.contains_key("content-type") {
            self.headers
                .insert("content-type".to_string(), "text/plain; charset=utf-8".to_string());
        }
        self.body = body.as_bytes().to_vec();
        self.sent = true;
    }

    /// Send a JSON response body. Automatically sets `Content-Type: application/json`.
    ///
    /// # Example
    /// ```
    /// use mini_http::response::Response;
    /// use serde_json::json;
    ///
    /// let mut res = Response::new();
    /// res.json(&json!({"status": "ok"}));
    /// ```
    pub fn json<T: serde::Serialize>(&mut self, value: &T) {
        self.headers
            .insert("content-type".to_string(), "application/json; charset=utf-8".to_string());
        match serde_json::to_vec(value) {
            Ok(bytes) => self.body = bytes,
            Err(e) => {
                self.status_code = 500;
                self.body = format!(r#"{{"error":"JSON serialization failed: {}"}}"#, e)
                    .into_bytes();
            }
        }
        self.sent = true;
    }

    /// Send an HTML response body. Automatically sets `Content-Type: text/html`.
    pub fn html(&mut self, body: &str) {
        self.headers
            .insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
        self.body = body.as_bytes().to_vec();
        self.sent = true;
    }

    /// Send a file as the response body. Automatically detects Content-Type from extension.
    ///
    /// # Example
    /// ```no_run
    /// use mini_http::response::Response;
    ///
    /// let mut res = Response::new();
    /// res.send_file("public/index.html");
    /// ```
    pub fn send_file(&mut self, path: &str) {
        let file_path = Path::new(path);

        match fs::read(file_path) {
            Ok(contents) => {
                // Guess MIME type from file extension
                let mime = mime_guess::from_path(file_path)
                    .first_or_octet_stream()
                    .to_string();
                self.headers.insert("content-type".to_string(), mime);
                self.body = contents;
            }
            Err(_) => {
                self.status_code = 404;
                self.headers
                    .insert("content-type".to_string(), "text/plain".to_string());
                self.body = b"File not found".to_vec();
            }
        }
        self.sent = true;
    }

    /// Send a redirect response.
    ///
    /// Uses 302 Found by default. Use `.status(301)` before calling for permanent redirect.
    pub fn redirect(&mut self, url: &str) {
        if self.status_code == 200 {
            self.status_code = 302;
        }
        self.headers.insert("location".to_string(), url.to_string());
        self.body = Vec::new();
        self.sent = true;
    }

    /// Set a cookie on the response.
    pub fn set_cookie(&mut self, cookie: CookieBuilder) {
        self.set_cookies.push(cookie.build());
    }

    /// Remove a cookie by setting it with Max-Age=0.
    pub fn remove_cookie(&mut self, name: &str) {
        self.set_cookies.push(CookieBuilder::removal(name).build());
    }

    /// Send raw bytes as the response body.
    pub fn send_bytes(&mut self, bytes: Vec<u8>) {
        if !self.headers.contains_key("content-type") {
            self.headers
                .insert("content-type".to_string(), "application/octet-stream".to_string());
        }
        self.body = bytes;
        self.sent = true;
    }

    /// Send an empty response with just a status code (e.g., 204 No Content).
    pub fn send_status(&mut self, code: u16) {
        self.status_code = code;
        self.body = Vec::new();
        self.sent = true;
    }

    /// Format the response as raw HTTP bytes for writing to the stream.
    pub fn to_bytes(&self) -> Vec<u8> {
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code,
            status_text(self.status_code)
        );

        let mut header_str = String::new();

        // Add Content-Length if not explicitly set
        let mut has_content_length = false;
        for (key, value) in &self.headers {
            header_str.push_str(&format!("{}: {}\r\n", key, value));
            if key.to_lowercase() == "content-length" {
                has_content_length = true;
            }
        }

        if !has_content_length {
            header_str.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }

        // Add Set-Cookie headers
        for cookie in &self.set_cookies {
            header_str.push_str(&format!("Set-Cookie: {}\r\n", cookie));
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(status_line.as_bytes());
        bytes.extend_from_slice(header_str.as_bytes());
        bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(&self.body);

        bytes
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_response() {
        let mut res = Response::new();
        res.send("Hello");
        assert_eq!(res.status_code, 200);
        assert_eq!(res.body, b"Hello");
        assert!(res.headers.get("content-type").unwrap().contains("text/plain"));
    }

    #[test]
    fn test_json_response() {
        let mut res = Response::new();
        res.json(&serde_json::json!({"ok": true}));
        assert_eq!(res.status_code, 200);
        assert!(res.headers.get("content-type").unwrap().contains("application/json"));
        let body: serde_json::Value = serde_json::from_slice(&res.body).unwrap();
        assert_eq!(body["ok"], true);
    }

    #[test]
    fn test_html_response() {
        let mut res = Response::new();
        res.html("<h1>Hello</h1>");
        assert!(res.headers.get("content-type").unwrap().contains("text/html"));
    }

    #[test]
    fn test_redirect() {
        let mut res = Response::new();
        res.redirect("/login");
        assert_eq!(res.status_code, 302);
        assert_eq!(res.headers.get("location").unwrap(), "/login");
    }

    #[test]
    fn test_status_chain() {
        let mut res = Response::new();
        res.status(201).send("Created");
        assert_eq!(res.status_code, 201);
    }

    #[test]
    fn test_to_bytes() {
        let mut res = Response::new();
        res.send("OK");
        let bytes = res.to_bytes();
        let response_str = String::from_utf8_lossy(&bytes);
        assert!(response_str.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(response_str.contains("Content-Length: 2"));
        assert!(response_str.ends_with("OK"));
    }
}
