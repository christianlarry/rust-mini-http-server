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

use crate::app::App;

fn main() {

    // Load environment variables from .env file
    dotenv().expect("Gagal load .env file");
    let port: String = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let host: String = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let mut app = App::new();

    app.get("/test", |req, res| {

        // Log Debug req params
        match &req.body {
            request::Body::Text(text) => println!("Request Body: {}", text),
            request::Body::Json(json) => println!("Request Body: {}", json),
            request::Body::Form(form_data) => println!("Request Form Data: {:?}", form_data),
            request::Body::Empty => println!("Request Body is Empty"),
        }

        println!("Query Params: {:?}", req.params);

        res.send("Home");
    });

    app.run(&format!("{}:{}", host, port));
}
