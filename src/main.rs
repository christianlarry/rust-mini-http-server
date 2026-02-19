//! Example: Mini HTTP server demonstrating framework features.

use mini_http::prelude::*;

fn main() {
    let mut app = App::new();

    // ── Middleware ────────────────────────────────────────────────
    app.use_middleware(Logger::new());
    app.use_middleware(Cors::permissive());

    // ── Basic Routes ─────────────────────────────────────────────
    app.get("/", |_req, res| {
        res.send("Welcome to Mini HTTP Framework! 🦀");
    });

    app.get("/hello/:name", |req, res| {
        let name = req.param("name").unwrap_or("World");
        res.json(&json!({
            "message": format!("Hello, {}!", name)
        }));
    });

    // ── JSON API ─────────────────────────────────────────────────
    app.get("/api/users", |_req, res| {
        res.json(&json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"},
            ]
        }));
    });

    app.get("/api/users/:id", |req, res| {
        let id = req.param("id").unwrap_or("0");
        res.json(&json!({
            "id": id,
            "name": "Alice",
            "email": "alice@example.com"
        }));
    });

    app.post("/api/users", |req, res| {
        let body = req.text();
        res.status(201).json(&json!({
            "message": "User created",
            "body": body
        }));
    });

    app.put("/api/users/:id", |req, res| {
        let id = req.param("id").unwrap_or("0");
        let body = req.text();
        res.json(&json!({
            "message": format!("User {} updated", id),
            "body": body
        }));
    });

    app.delete("/api/users/:id", |req, res| {
        let id = req.param("id").unwrap_or("0");
        res.json(&json!({
            "message": format!("User {} deleted", id)
        }));
    });

    // ── HTML Response ────────────────────────────────────────────
    app.get("/page", |_req, res| {
        res.html(
            r#"<!DOCTYPE html>
<html>
<head><title>Mini HTTP</title></head>
<body>
    <h1>Welcome to Mini HTTP Framework</h1>
    <p>Built with Rust 🦀</p>
</body>
</html>"#,
        );
    });

    // ── Query Parameters ─────────────────────────────────────────
    app.get("/search", |req, res| {
        let query = req.query_param("q").unwrap_or("(none)");
        let page = req.query_param("page").unwrap_or("1");
        res.json(&json!({
            "query": query,
            "page": page,
        }));
    });

    // ── Redirect ─────────────────────────────────────────────────
    app.get("/old-page", |_req, res| {
        res.redirect("/page");
    });

    // ── Route Group ──────────────────────────────────────────────
    let mut api_v2 = RouteGroup::new("/api/v2");
    api_v2.get("/status", |_req, res| {
        res.json(&json!({"version": "2.0", "status": "ok"}));
    });
    app.group(api_v2);

    // ── Start Server ─────────────────────────────────────────────
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    app.listen(&format!("{}:{}", host, port));
}
