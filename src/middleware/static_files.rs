//! Static file serving middleware.
//!
//! Serves files from a local directory, mapping URL paths to filesystem paths.
//! Automatically detects MIME types from file extensions.

use std::fs;
use std::path::PathBuf;

use crate::middleware::Middleware;
use crate::request::{Method, Request};
use crate::response::Response;

/// Static file serving middleware.
///
/// Maps a URL prefix to a filesystem directory and serves files with
/// appropriate MIME types.
///
/// # Example
/// ```no_run
/// use mini_http::middleware::static_files::StaticFiles;
///
/// // Serve files from ./public at /static/*
/// let statics = StaticFiles::new("/static", "./public");
///
/// // Serve with index.html fallback
/// let statics = StaticFiles::new("/", "./public").index_file("index.html");
/// ```
pub struct StaticFiles {
    /// URL prefix to match.
    url_prefix: String,
    /// Filesystem directory to serve from.
    root_dir: PathBuf,
    /// Default index file name.
    index_file: Option<String>,
    /// Cache-Control header value.
    cache_control: Option<String>,
}

impl StaticFiles {
    /// Create static file middleware mapping `url_prefix` to `root_dir`.
    pub fn new(url_prefix: &str, root_dir: &str) -> Self {
        StaticFiles {
            url_prefix: url_prefix.trim_end_matches('/').to_string(),
            root_dir: PathBuf::from(root_dir),
            index_file: None,
            cache_control: None,
        }
    }

    /// Set a default index file (e.g., `index.html`) for directory requests.
    pub fn index_file(mut self, filename: &str) -> Self {
        self.index_file = Some(filename.to_string());
        self
    }

    /// Set the Cache-Control header for served files.
    pub fn cache_control(mut self, value: &str) -> Self {
        self.cache_control = Some(value.to_string());
        self
    }

    /// Resolve a URL path to a safe filesystem path.
    fn resolve_path(&self, url_path: &str) -> Option<PathBuf> {
        // Strip the URL prefix
        let relative = if url_path == self.url_prefix || url_path == format!("{}/", self.url_prefix)
        {
            String::new()
        } else if let Some(stripped) = url_path.strip_prefix(&format!("{}/", self.url_prefix)) {
            stripped.to_string()
        } else if let Some(stripped) = url_path.strip_prefix(&self.url_prefix) {
            stripped.to_string()
        } else {
            return None;
        };

        // Security: prevent directory traversal
        let relative = relative.replace('\\', "/");
        if relative.contains("..") {
            return None;
        }

        let mut file_path = self.root_dir.clone();
        if relative.is_empty() {
            // Try index file for directory
            if let Some(ref index) = self.index_file {
                file_path.push(index);
            } else {
                return None;
            }
        } else {
            file_path.push(&relative);
        }

        // If path is a directory, try index file
        if file_path.is_dir() {
            if let Some(ref index) = self.index_file {
                file_path.push(index);
            } else {
                return None;
            }
        }

        // Verify the resolved path is within the root directory
        let canonical_root = self.root_dir.canonicalize().ok()?;
        let canonical_file = file_path.canonicalize().ok()?;
        if !canonical_file.starts_with(&canonical_root) {
            return None;
        }

        Some(file_path)
    }
}

impl Middleware for StaticFiles {
    fn before(&self, req: &mut Request, res: &mut Response) -> bool {
        // Only handle GET and HEAD requests
        if req.method != Method::GET && req.method != Method::HEAD {
            return true;
        }

        // Only handle requests matching our prefix
        if !req.path.starts_with(&self.url_prefix) {
            return true;
        }

        // Try to resolve the file path
        if let Some(file_path) = self.resolve_path(&req.path) {
            if let Ok(contents) = fs::read(&file_path) {
                let mime = mime_guess::from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();

                res.header("content-type", &mime);

                if let Some(ref cache) = self.cache_control {
                    res.header("cache-control", cache);
                }

                // Add Last-Modified header
                if let Ok(metadata) = fs::metadata(&file_path) {
                    if let Ok(modified) = metadata.modified() {
                        let datetime: chrono::DateTime<chrono::Utc> = modified.into();
                        res.header(
                            "last-modified",
                            &datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
                        );
                    }
                }

                if req.method == Method::HEAD {
                    // HEAD: just headers, no body
                    res.header("content-length", &contents.len().to_string());
                    res.send_status(200);
                } else {
                    res.send_bytes(contents);
                }

                return false; // Don't continue to route handlers
            }
        }

        true // File not found, continue to route handlers
    }

    fn name(&self) -> &str {
        "static_files"
    }
}
