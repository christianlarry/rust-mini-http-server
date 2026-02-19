//! Rate limiting middleware.
//!
//! Implements a fixed-window rate limiter that tracks requests per IP address.
//! Requests exceeding the limit receive a `429 Too Many Requests` response.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::Response;

/// Tracks request counts within a time window.
struct WindowCounter {
    count: u64,
    window_start: Instant,
}

/// Rate limiting middleware using a fixed-window counter per IP address.
///
/// # Example
/// ```
/// use mini_http::middleware::rate_limiter::RateLimiter;
/// use std::time::Duration;
///
/// // Allow 100 requests per minute per IP
/// let limiter = RateLimiter::new(100, Duration::from_secs(60));
///
/// // Or use the builder
/// let limiter = RateLimiter::per_minute(60);
/// ```
pub struct RateLimiter {
    max_requests: u64,
    window_duration: Duration,
    counters: Arc<Mutex<HashMap<String, WindowCounter>>>,
}

impl RateLimiter {
    /// Create a rate limiter with custom max requests and window duration.
    pub fn new(max_requests: u64, window_duration: Duration) -> Self {
        RateLimiter {
            max_requests,
            window_duration,
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a rate limiter allowing N requests per second.
    pub fn per_second(max: u64) -> Self {
        Self::new(max, Duration::from_secs(1))
    }

    /// Create a rate limiter allowing N requests per minute.
    pub fn per_minute(max: u64) -> Self {
        Self::new(max, Duration::from_secs(60))
    }

    /// Create a rate limiter allowing N requests per hour.
    pub fn per_hour(max: u64) -> Self {
        Self::new(max, Duration::from_secs(3600))
    }

    /// Get the client identifier (IP address).
    fn client_key(req: &Request) -> String {
        req.remote_addr
            .map(|addr| addr.ip().to_string())
            .or_else(|| req.header("x-forwarded-for").map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Clean up expired window entries.
    pub fn cleanup(&self) {
        if let Ok(mut counters) = self.counters.lock() {
            counters.retain(|_, counter| counter.window_start.elapsed() < self.window_duration);
        }
    }
}

impl Middleware for RateLimiter {
    fn before(&self, req: &mut Request, res: &mut Response) -> bool {
        let key = Self::client_key(req);
        let now = Instant::now();

        let mut counters = self.counters.lock().unwrap();

        let counter = counters.entry(key).or_insert_with(|| WindowCounter {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if counter.window_start.elapsed() >= self.window_duration {
            counter.count = 0;
            counter.window_start = now;
        }

        counter.count += 1;

        let remaining = self.max_requests.saturating_sub(counter.count);
        let reset = counter.window_start + self.window_duration;
        let reset_secs = reset
            .checked_duration_since(now)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        // Set rate limit headers
        res.header("X-RateLimit-Limit", &self.max_requests.to_string());
        res.header("X-RateLimit-Remaining", &remaining.to_string());
        res.header("X-RateLimit-Reset", &reset_secs.to_string());

        if counter.count > self.max_requests {
            res.header("Retry-After", &reset_secs.to_string());
            res.status(429).json(&serde_json::json!({
                "error": {
                    "code": 429,
                    "message": "Too Many Requests",
                    "retry_after": reset_secs
                }
            }));
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "rate_limiter"
    }
}
