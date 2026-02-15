// Learning Rust For the first time. I'm gonna try every API in Rust and see how it works. I hope I can learn it fast and use it for my projects.
// Today im gonna make Simple HTTP Server using Rust.

use std::net::{TcpListener};

// Module untuk server
mod server;

fn main() {
    // 1️⃣ Bind server ke address dan port
    // Ini bikin server "listen" di localhost port 7878
    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("Gagal bind port");

    println!("Server jalan di http://127.0.0.1:7878");

    // 2️⃣ Loop untuk nerima koneksi masuk
    for stream in listener.incoming() {

        // 3️⃣ Ambil koneksi TCP dari client
        match stream {
            Ok(stream) => {
                // 4️⃣ Handle koneksi di fungsi terpisah
                server::handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Gagal menerima koneksi: {}", e);
            }
        }
    }
}
