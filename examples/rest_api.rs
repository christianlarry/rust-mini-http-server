//! REST API example demonstrating CRUD operations with JSON.

use mini_http::prelude::*;
use mini_http::context::new_shared_state;

fn main() {
    let mut app = App::new();

    // Shared state to store users
    let state = new_shared_state();
    {
        let mut s = state.write().unwrap();
        s.insert("users", Vec::<serde_json::Value>::new());
    }
    app.set_state(state);

    // Middleware
    app.use_middleware(Logger::new());
    app.use_middleware(Cors::permissive());

    // ── Routes ───────────────────────────────────────────────────

    // List all users
    app.get("/api/users", |req, res| {
        if let Some(state) = req.app_state() {
            let s = state.read().unwrap();
            if let Some(users) = s.get::<Vec<serde_json::Value>>("users") {
                res.json(&json!({ "users": users }));
                return;
            }
        }
        res.json(&json!({ "users": [] }));
    });

    // Get user by ID
    app.get("/api/users/:id", |req, res| {
        let id = req.param("id").unwrap_or("0");
        if let Some(state) = req.app_state() {
            let s = state.read().unwrap();
            if let Some(users) = s.get::<Vec<serde_json::Value>>("users") {
                if let Some(user) = users.iter().find(|u| u["id"].to_string().trim_matches('"') == id) {
                    res.json(user);
                    return;
                }
            }
        }
        res.status(404).json(&json!({
            "error": format!("User {} not found", id)
        }));
    });

    // Create user
    app.post("/api/users", |req, res| {
        let body: serde_json::Value = match req.json() {
            Ok(v) => v,
            Err(e) => {
                res.status(400).json(&json!({"error": e.to_string()}));
                return;
            }
        };

        if let Some(state) = req.app_state() {
            let mut s = state.write().unwrap();
            if let Some(users) = s.get_mut::<Vec<serde_json::Value>>("users") {
                let id = users.len() + 1;
                let mut user = body;
                user["id"] = json!(id);
                users.push(user.clone());
                res.status(201).json(&user);
                return;
            }
        }
        res.status(500).json(&json!({"error": "Internal error"}));
    });

    // Update user
    app.put("/api/users/:id", |req, res| {
        let id = req.param("id").unwrap_or("0").to_string();
        let body: serde_json::Value = match req.json() {
            Ok(v) => v,
            Err(e) => {
                res.status(400).json(&json!({"error": e.to_string()}));
                return;
            }
        };

        if let Some(state) = req.app_state() {
            let mut s = state.write().unwrap();
            if let Some(users) = s.get_mut::<Vec<serde_json::Value>>("users") {
                if let Some(user) = users.iter_mut().find(|u| u["id"].to_string().trim_matches('"') == id) {
                    for (key, value) in body.as_object().unwrap_or(&serde_json::Map::new()) {
                        user[key] = value.clone();
                    }
                    res.json(&*user);
                    return;
                }
            }
        }
        res.status(404).json(&json!({"error": format!("User {} not found", id)}));
    });

    // Delete user
    app.delete("/api/users/:id", |req, res| {
        let id = req.param("id").unwrap_or("0").to_string();

        if let Some(state) = req.app_state() {
            let mut s = state.write().unwrap();
            if let Some(users) = s.get_mut::<Vec<serde_json::Value>>("users") {
                let before = users.len();
                users.retain(|u| u["id"].to_string().trim_matches('"') != id);
                if users.len() < before {
                    res.json(&json!({"message": format!("User {} deleted", id)}));
                    return;
                }
            }
        }
        res.status(404).json(&json!({"error": format!("User {} not found", id)}));
    });

    // ── Start ────────────────────────────────────────────────────
    println!("REST API running at http://127.0.0.1:3000");
    app.listen("127.0.0.1:3000");
}
