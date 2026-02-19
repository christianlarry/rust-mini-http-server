//! HTTP request parsing and representation.
//!
//! Provides the [`Request`] struct for representing parsed HTTP requests,
//! the [`Method`] enum for HTTP methods, and the [`Body`] enum for parsed
//! request bodies. Uses the `httparse` crate for robust HTTP/1.1 parsing.

use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;

use serde::de::DeserializeOwned;

use crate::context::SharedState;
use crate::error::{Error, Result};

/// HTTP methods supported by the framework.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    /// Returns the method as a static string slice.
    pub fn as_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::CONNECT => "CONNECT",
            Method::TRACE => "TRACE",
        }
    }

    /// Parse a method string into a `Method` enum.
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "CONNECT" => Method::CONNECT,
            "TRACE" => Method::TRACE,
            _ => Method::GET,
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents the parsed body of an HTTP request.
#[derive(Debug, Clone)]
pub enum Body {
    /// Plain text body.
    Text(String),
    /// Parsed JSON value.
    Json(serde_json::Value),
    /// URL-encoded form data.
    Form(HashMap<String, String>),
    /// Multipart form data with file uploads.
    Multipart(Vec<MultipartField>),
    /// Raw binary data.
    Binary(Vec<u8>),
    /// No body present.
    Empty,
}

/// Represents a single field in a multipart form submission.
#[derive(Debug, Clone)]
pub struct MultipartField {
    /// Field name from Content-Disposition.
    pub name: String,
    /// Original filename (for file uploads).
    pub filename: Option<String>,
    /// MIME type of the field content.
    pub content_type: Option<String>,
    /// Raw field data.
    pub data: Vec<u8>,
}

/// Represents a fully parsed HTTP request.
///
/// Created by the framework's request parser and passed to route handlers
/// and middleware. Contains all parsed components of the HTTP request
/// including headers, query parameters, path parameters, body, and cookies.
#[derive(Debug)]
pub struct Request {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: Method,
    /// Request path without query string (e.g., `/users/123`).
    pub path: String,
    /// Full request URI including query string.
    pub uri: String,
    /// HTTP version string (e.g., `HTTP/1.1`).
    pub http_version: String,
    /// Request headers (lowercase keys).
    pub headers: HashMap<String, String>,
    /// Query parameters (supports multiple values per key).
    pub query: HashMap<String, Vec<String>>,
    /// Path parameters extracted from route patterns (e.g., `:id`).
    pub params: HashMap<String, String>,
    /// Parsed request body.
    pub body: Body,
    /// Raw body bytes.
    pub raw_body: Vec<u8>,
    /// Parsed cookies from the `Cookie` header.
    pub cookies: HashMap<String, String>,
    /// Remote address of the client.
    pub remote_addr: Option<SocketAddr>,
    /// Per-request extensions for middleware data sharing.
    pub extensions: HashMap<String, Box<dyn Any + Send + Sync>>,
    /// Shared application state (set by the server).
    pub state: Option<SharedState>,
}

impl Request {
    /// Parse raw bytes into an HTTP Request using `httparse`.
    ///
    /// # Errors
    /// Returns `Error::Parse` if the HTTP request is malformed or incomplete.
    pub fn parse(buffer: &[u8]) -> Result<Self> {
        let mut headers_buf = [httparse::EMPTY_HEADER; 128];
        let mut req = httparse::Request::new(&mut headers_buf);

        let body_offset = match req.parse(buffer) {
            Ok(httparse::Status::Complete(offset)) => offset,
            Ok(httparse::Status::Partial) => {
                return Err(Error::Parse("Incomplete HTTP request".into()));
            }
            Err(e) => return Err(Error::Parse(e.to_string())),
        };

        let method = Method::from_str(req.method.unwrap_or("GET"));
        let full_path = req.path.unwrap_or("/").to_string();
        let http_version = format!("HTTP/1.{}", req.version.unwrap_or(1));

        // Parse headers into HashMap with lowercase keys
        let mut headers = HashMap::new();
        for header in req.headers.iter() {
            let name = header.name.to_lowercase();
            let value = String::from_utf8_lossy(header.value).to_string();
            headers.insert(name, value);
        }

        // Parse URL path and query parameters
        let (path, query) = parse_url(&full_path);

        // Parse body based on Content-Length
        let content_length: usize = headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let raw_body = if content_length > 0 && body_offset < buffer.len() {
            let end = std::cmp::min(body_offset + content_length, buffer.len());
            buffer[body_offset..end].to_vec()
        } else {
            Vec::new()
        };

        let body = parse_body(&headers, &raw_body);
        let cookies = parse_cookies(&headers);

        Ok(Request {
            method,
            path,
            uri: full_path,
            http_version,
            headers,
            query,
            params: HashMap::new(),
            body,
            raw_body,
            cookies,
            remote_addr: None,
            extensions: HashMap::new(),
            state: None,
        })
    }

    /// Get a single query parameter value.
    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query
            .get(key)
            .and_then(|v| v.first().map(|s| s.as_str()))
    }

    /// Get all values for a query parameter.
    pub fn query_params(&self, key: &str) -> Option<&Vec<String>> {
        self.query.get(key)
    }

    /// Get a path parameter value (from route patterns like `:id`).
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Get a header value (case-insensitive).
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(&key.to_lowercase()).map(|s| s.as_str())
    }

    /// Get a cookie value by name.
    pub fn cookie(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(|s| s.as_str())
    }

    /// Deserialize the JSON body into a typed struct.
    ///
    /// # Errors
    /// Returns `Error::Json` if deserialization fails, or `Error::BadRequest`
    /// if no JSON body is present.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        match &self.body {
            Body::Json(value) => serde_json::from_value(value.clone()).map_err(Error::Json),
            _ => {
                if !self.raw_body.is_empty() {
                    serde_json::from_slice(&self.raw_body).map_err(Error::Json)
                } else {
                    Err(Error::BadRequest("No JSON body found".into()))
                }
            }
        }
    }

    /// Get the body as a text string.
    pub fn text(&self) -> String {
        match &self.body {
            Body::Text(s) => s.clone(),
            Body::Json(v) => v.to_string(),
            _ => String::from_utf8_lossy(&self.raw_body).to_string(),
        }
    }

    /// Get a form field value from URL-encoded body.
    pub fn form_field(&self, key: &str) -> Option<&str> {
        match &self.body {
            Body::Form(map) => map.get(key).map(|s| s.as_str()),
            _ => None,
        }
    }

    /// Get multipart form fields.
    pub fn multipart_fields(&self) -> Option<&Vec<MultipartField>> {
        match &self.body {
            Body::Multipart(fields) => Some(fields),
            _ => None,
        }
    }

    /// Get a specific multipart field by name.
    pub fn multipart_field(&self, name: &str) -> Option<&MultipartField> {
        match &self.body {
            Body::Multipart(fields) => fields.iter().find(|f| f.name == name),
            _ => None,
        }
    }

    /// Get the Content-Type header value.
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Check if the request `Accept` header includes a specific content type.
    pub fn accepts(&self, content_type: &str) -> bool {
        self.header("accept")
            .is_some_and(|accept| accept.contains(content_type) || accept.contains("*/*"))
    }

    /// Check if this is a WebSocket upgrade request.
    pub fn is_websocket_upgrade(&self) -> bool {
        self.header("upgrade")
            .is_some_and(|v| v.eq_ignore_ascii_case("websocket"))
            && self.header("connection")
                .is_some_and(|v| v.to_lowercase().contains("upgrade"))
    }

    /// Set a per-request extension value for middleware data sharing.
    pub fn set_extension<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.extensions.insert(key.to_string(), Box::new(value));
    }

    /// Get a per-request extension value.
    pub fn get_extension<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.extensions.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Get a mutable reference to a per-request extension value.
    pub fn get_extension_mut<T: Any + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.extensions
            .get_mut(key)
            .and_then(|v| v.downcast_mut::<T>())
    }

    /// Access the shared application state.
    pub fn app_state(&self) -> Option<&SharedState> {
        self.state.as_ref()
    }
}

// Provide a Debug-safe representation (extensions aren't Debug by default)
impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.path, self.http_version)
    }
}

/// Parse URL path and query string using `urlencoding` for percent-decoding.
fn parse_url(full_path: &str) -> (String, HashMap<String, Vec<String>>) {
    let mut query = HashMap::new();

    if let Some(pos) = full_path.find('?') {
        let path = full_path[..pos].to_string();
        let query_string = &full_path[pos + 1..];

        for pair in query_string.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut kv = pair.splitn(2, '=');
            if let Some(key) = kv.next() {
                let key = urlencoding::decode(key).unwrap_or_default().to_string();
                let value = kv
                    .next()
                    .map(|v| urlencoding::decode(v).unwrap_or_default().to_string())
                    .unwrap_or_default();
                query.entry(key).or_insert_with(Vec::new).push(value);
            }
        }
        (path, query)
    } else {
        (full_path.to_string(), query)
    }
}

/// Parse the request body based on Content-Type header.
fn parse_body(headers: &HashMap<String, String>, raw_body: &[u8]) -> Body {
    if raw_body.is_empty() {
        return Body::Empty;
    }

    let content_type = headers
        .get("content-type")
        .map(|s| s.as_str())
        .unwrap_or("");

    if content_type.starts_with("application/json") {
        match serde_json::from_slice(raw_body) {
            Ok(json) => Body::Json(json),
            Err(_) => Body::Text(String::from_utf8_lossy(raw_body).to_string()),
        }
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        let body_str = String::from_utf8_lossy(raw_body);
        let mut form_data = HashMap::new();
        for pair in body_str.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut kv = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                form_data.insert(
                    urlencoding::decode(key).unwrap_or_default().to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                );
            }
        }
        Body::Form(form_data)
    } else if content_type.starts_with("multipart/form-data") {
        parse_multipart(content_type, raw_body)
    } else if content_type.starts_with("text/") {
        Body::Text(String::from_utf8_lossy(raw_body).to_string())
    } else {
        Body::Binary(raw_body.to_vec())
    }
}

/// Parse multipart/form-data body.
fn parse_multipart(content_type: &str, body: &[u8]) -> Body {
    let boundary = content_type
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("boundary=") {
                Some(rest.trim_matches('"').to_string())
            } else {
                None
            }
        });

    let boundary = match boundary {
        Some(b) => b,
        None => return Body::Binary(body.to_vec()),
    };

    let body_str = String::from_utf8_lossy(body);
    let delimiter = format!("--{}", boundary);

    let mut fields = Vec::new();

    for part in body_str.split(&delimiter) {
        let part = part.trim();
        if part.is_empty() || part == "--" {
            continue;
        }

        // Split headers from body at double CRLF
        if let Some(header_end) = part.find("\r\n\r\n") {
            let headers_section = &part[..header_end];
            let data = part[header_end + 4..].trim_end_matches("\r\n");

            let mut name = String::new();
            let mut filename = None;
            let mut field_content_type = None;

            for line in headers_section.lines() {
                let line = line.trim();
                if line.to_lowercase().starts_with("content-disposition:") {
                    for param in line.split(';') {
                        let param = param.trim();
                        if let Some(n) = param.strip_prefix("name=") {
                            name = n.trim_matches('"').to_string();
                        } else if let Some(n) = param.strip_prefix("filename=") {
                            filename = Some(n.trim_matches('"').to_string());
                        }
                    }
                } else if line.to_lowercase().starts_with("content-type:") {
                    field_content_type = Some(line[13..].trim().to_string());
                }
            }

            if !name.is_empty() {
                fields.push(MultipartField {
                    name,
                    filename,
                    content_type: field_content_type,
                    data: data.as_bytes().to_vec(),
                });
            }
        }
    }

    Body::Multipart(fields)
}

/// Parse cookies from the `Cookie` header.
fn parse_cookies(headers: &HashMap<String, String>) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    if let Some(cookie_header) = headers.get("cookie") {
        for cookie_str in cookie_header.split(';') {
            let cookie_str = cookie_str.trim();
            if let Some(pos) = cookie_str.find('=') {
                let name = cookie_str[..pos].trim().to_string();
                let value = cookie_str[pos + 1..].trim().to_string();
                cookies.insert(name, value);
            }
        }
    }
    cookies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let raw = b"GET /hello?name=world HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::parse(raw).unwrap();
        assert_eq!(req.method, Method::GET);
        assert_eq!(req.path, "/hello");
        assert_eq!(req.query_param("name"), Some("world"));
        assert_eq!(req.http_version, "HTTP/1.1");
    }

    #[test]
    fn test_parse_post_json() {
        let body = r#"{"name":"test"}"#;
        let raw = format!(
            "POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let req = Request::parse(raw.as_bytes()).unwrap();
        assert_eq!(req.method, Method::POST);
        assert_eq!(req.path, "/api/users");
        match &req.body {
            Body::Json(v) => assert_eq!(v["name"], "test"),
            _ => panic!("Expected JSON body"),
        }
    }

    #[test]
    fn test_parse_form_body() {
        let body = "username=john&password=secret";
        let raw = format!(
            "POST /login HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let req = Request::parse(raw.as_bytes()).unwrap();
        assert_eq!(req.form_field("username"), Some("john"));
        assert_eq!(req.form_field("password"), Some("secret"));
    }

    #[test]
    fn test_parse_cookies() {
        let raw = b"GET / HTTP/1.1\r\nCookie: session=abc123; theme=dark\r\n\r\n";
        let req = Request::parse(raw).unwrap();
        assert_eq!(req.cookie("session"), Some("abc123"));
        assert_eq!(req.cookie("theme"), Some("dark"));
    }

    #[test]
    fn test_parse_url_with_multiple_query_params() {
        let raw = b"GET /search?tag=rust&tag=web&page=1 HTTP/1.1\r\n\r\n";
        let req = Request::parse(raw).unwrap();
        assert_eq!(req.path, "/search");
        let tags = req.query_params("tag").unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(req.query_param("page"), Some("1"));
    }

    #[test]
    fn test_method_display() {
        assert_eq!(Method::GET.to_string(), "GET");
        assert_eq!(Method::POST.to_string(), "POST");
        assert_eq!(Method::DELETE.to_string(), "DELETE");
    }
}
