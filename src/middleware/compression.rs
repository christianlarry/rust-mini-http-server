//! Response compression middleware (Gzip).
//!
//! Automatically compresses response bodies using Gzip when the client
//! indicates support via the `Accept-Encoding` header.

use std::io::Write;

use flate2::write::GzEncoder;
use flate2::Compression;

use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;

/// Compression middleware that applies Gzip encoding to response bodies.
///
/// Only compresses responses that:
/// - Have a body larger than the minimum size threshold
/// - Are text-based content types (text/*, application/json, application/javascript, etc.)
/// - Are requested by a client that accepts gzip encoding
///
/// # Example
/// ```
/// use mini_http::middleware::compression::CompressionMiddleware;
///
/// let compression = CompressionMiddleware::new();
/// // or with custom minimum size (default 1024 bytes):
/// let compression = CompressionMiddleware::with_min_size(512);
/// ```
pub struct CompressionMiddleware {
    min_size: usize,
    level: Compression,
}

impl CompressionMiddleware {
    /// Create compression middleware with default settings (min 1024 bytes, default level).
    pub fn new() -> Self {
        CompressionMiddleware {
            min_size: 1024,
            level: Compression::default(),
        }
    }

    /// Create compression middleware with a custom minimum body size.
    pub fn with_min_size(min_size: usize) -> Self {
        CompressionMiddleware {
            min_size,
            level: Compression::default(),
        }
    }

    /// Set the compression level (0-9, where 9 is maximum compression).
    pub fn level(mut self, level: u32) -> Self {
        self.level = Compression::new(level);
        self
    }

    /// Check if the content type is compressible.
    fn is_compressible(content_type: &str) -> bool {
        content_type.starts_with("text/")
            || content_type.contains("json")
            || content_type.contains("javascript")
            || content_type.contains("xml")
            || content_type.contains("svg")
            || content_type.contains("html")
    }
}

impl Default for CompressionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for CompressionMiddleware {
    fn before(&self, _req: &mut Request, _res: &mut Response) -> bool {
        true
    }

    fn after(&self, req: &mut Request, res: &mut Response) {
        // Only compress if client accepts gzip
        let accepts_gzip = req
            .header("accept-encoding")
            .is_some_and(|ae| ae.contains("gzip"));

        if !accepts_gzip {
            return;
        }

        // Don't compress small bodies
        if res.body.len() < self.min_size {
            return;
        }

        // Only compress compressible content types
        let content_type = res
            .headers
            .get("content-type")
            .cloned()
            .unwrap_or_default();

        if !Self::is_compressible(&content_type) {
            return;
        }

        // Don't re-compress already compressed content
        if res.headers.contains_key("content-encoding") {
            return;
        }

        // Compress the body
        let mut encoder = GzEncoder::new(Vec::new(), self.level);
        if encoder.write_all(&res.body).is_ok() {
            if let Ok(compressed) = encoder.finish() {
                if compressed.len() < res.body.len() {
                    res.body = compressed;
                    res.header("Content-Encoding", "gzip");
                    res.header("Vary", "Accept-Encoding");
                }
            }
        }
    }

    fn name(&self) -> &str {
        "compression"
    }
}
