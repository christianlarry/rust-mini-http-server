// Learning Rust For the first time. I'm gonna try every API in Rust and see how it works. I hope I can learn it fast and use it for my projects.
// Today im gonna make Simple HTTP Server using Rust.
use std::env;
use dotenvy::dotenv;

// Module untuk server
mod server;
mod app;
mod request;
mod response;
mod router;

fn main() {

    // Load environment variables from .env file
    dotenv().expect("Gagal load .env file");
    let port: String = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let host: String = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let mut app = app::App::new();

    app.get("/", |_, res| {
        res.send("Home");
    });

    app.run(&format!("{}:{}", host, port));
}
