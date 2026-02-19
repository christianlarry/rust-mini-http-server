//! Request logging middleware.
//!
//! Logs incoming requests with timestamp, method, path, status code,
//! and response time. Output format is similar to Express.js's `morgan`.

use std::time::Instant;

use chrono::Local;

use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;

/// Log output format.
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// `[2025-01-15 10:30:00] GET /path -> 200 (5ms)`
    Default,
    /// Minimal: `GET /path 200 5ms`
    Minimal,
    /// Combined (Apache-like): includes remote addr and user agent
    Combined,
}

/// Logger middleware that writes request logs to stderr.
///
/// # Example
/// ```
/// use mini_http::middleware::logger::{Logger, LogFormat};
///
/// let logger = Logger::new();
/// // or with custom format:
/// let logger = Logger::with_format(LogFormat::Combined);
/// ```
pub struct Logger {
    format: LogFormat,
}

impl Logger {
    /// Create a logger with the default format.
    pub fn new() -> Self {
        Logger {
            format: LogFormat::Default,
        }
    }

    /// Create a logger with a specific format.
    pub fn with_format(format: LogFormat) -> Self {
        Logger { format }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Logger {
    fn before(&self, req: &mut Request, _res: &mut Response) -> bool {
        // Store the request start time as an extension
        req.set_extension("_log_start", Instant::now());
        true
    }

    fn after(&self, req: &mut Request, res: &mut Response) {
        let duration = req
            .get_extension::<Instant>("_log_start")
            .map(|start| start.elapsed())
            .unwrap_or_default();

        let duration_ms = duration.as_secs_f64() * 1000.0;

        match self.format {
            LogFormat::Default => {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                eprintln!(
                    "[{}] {} {} -> {} ({:.2}ms)",
                    timestamp,
                    req.method,
                    req.uri,
                    res.status_code,
                    duration_ms,
                );
            }
            LogFormat::Minimal => {
                eprintln!(
                    "{} {} {} {:.2}ms",
                    req.method, req.uri, res.status_code, duration_ms,
                );
            }
            LogFormat::Combined => {
                let timestamp = Local::now().format("%d/%b/%Y:%H:%M:%S %z");
                let remote = req
                    .remote_addr
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let user_agent = req.header("user-agent").unwrap_or("-");
                let referer = req.header("referer").unwrap_or("-");
                eprintln!(
                    "{} - - [{}] \"{} {} {}\" {} {} \"{}\" \"{}\" {:.2}ms",
                    remote,
                    timestamp,
                    req.method,
                    req.uri,
                    req.http_version,
                    res.status_code,
                    res.body.len(),
                    referer,
                    user_agent,
                    duration_ms,
                );
            }
        }
    }

    fn name(&self) -> &str {
        "logger"
    }
}
