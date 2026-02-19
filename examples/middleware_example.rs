//! Middleware example demonstrating logging, CORS, rate limiting, and compression.

use mini_http::prelude::*;
use mini_http::middleware::compression::CompressionMiddleware;
use mini_http::middleware::rate_limiter::RateLimiter;

fn main() {
    let mut app = App::new();

    // ── Middleware Stack ──────────────────────────────────────────
    // Middleware execute in registration order (before hooks)
    // and reverse order (after hooks).

    // 1. Logger — logs all requests with timing
    app.use_middleware(Logger::new());

    // 2. CORS — allow cross-origin requests
    app.use_middleware(Cors::permissive());

    // 3. Rate Limiter — 30 requests per minute per IP
    app.use_middleware(RateLimiter::per_minute(30));

    // 4. Compression — gzip responses larger than 512 bytes
    app.use_middleware(CompressionMiddleware::with_min_size(512));

    // ── Routes ───────────────────────────────────────────────────

    app.get("/", |_req, res| {
        res.send("Hello with middleware!");
    });

    app.get("/large", |_req, res| {
        // Generate a large response to trigger compression
        let data: String = (0..100)
            .map(|i| format!("Item {}: This is a line of data for testing compression.\n", i))
            .collect();
        res.header("Content-Type", "text/plain")
            .send(&data);
    });

    app.get("/api/data", |_req, res| {
        let items: Vec<serde_json::Value> = (0..50)
            .map(|i| json!({"id": i, "value": format!("item_{}", i)}))
            .collect();
        res.json(&json!({"data": items}));
    });

    // Custom middleware example: Auth check
    struct AuthGuard;
    impl Middleware for AuthGuard {
        fn before(&self, req: &mut Request, res: &mut Response) -> bool {
            if req.header("authorization").is_none() {
                res.status(401).json(&json!({
                    "error": "Authorization header required"
                }));
                return false;
            }
            true
        }
        fn name(&self) -> &str { "auth_guard" }
    }

    // Protected routes in a group
    let mut protected = RouteGroup::new("/admin");
    protected.get("/dashboard", |_req, res| {
        res.json(&json!({"page": "admin dashboard"}));
    });
    protected.get("/settings", |_req, res| {
        res.json(&json!({"page": "admin settings"}));
    });

    // Note: group-level middleware is applied via the global middleware stack
    // For route-specific auth, apply it globally or check inside handlers
    app.use_middleware(AuthGuard);
    app.group(protected);

    // ── Start ────────────────────────────────────────────────────
    println!("Middleware example running at http://127.0.0.1:3000");
    app.listen("127.0.0.1:3000");
}
