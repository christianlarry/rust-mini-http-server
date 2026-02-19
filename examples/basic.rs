//! Basic "Hello World" example demonstrating minimal framework usage.

use mini_http::prelude::*;

fn main() {
    let mut app = App::new();

    app.get("/", |_req, res| {
        res.send("Hello, World!");
    });

    app.get("/greet/:name", |req, res| {
        let name = req.param("name").unwrap_or("stranger");
        res.send(&format!("Hello, {}!", name));
    });

    println!("Starting server at http://127.0.0.1:3000");
    app.listen("127.0.0.1:3000");
}
