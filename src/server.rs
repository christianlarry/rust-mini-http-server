// src/server.rs
use std::net::TcpStream;
use std::io::{Read, Write};

fn parse_request(request: &str) -> Option<(&str, &str)> {
    // This function will parse the HTTP request and return the method and path.
    // For example, if the request is "GET / HTTP/1.1", it will return ("GET", "/").
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0];
    let path = if parts[1] == "/favicon.ico" {
        "/" // Redirect favicon request to root
    } else {
        parts[1]
    };

    Some((method, path))
}

pub fn handle_connection(mut stream: TcpStream){
    // This function will handle the incoming connection and send a response back to the client.
    // For now, we will just read the request and print it to the console.

    let mut buffer = [0; 1024]; // [0; 1024] artinya buat array dengan 1024 elemen// Baca data dari client

    // Kita gak pakai read_to_end karena HTTP keep-alive
    let bytes_read = stream.read(&mut buffer)
        .expect("Gagal baca request");

    // Convert request jadi string buat debug
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    let parsed_request = parse_request(&request);
    println!("Parsed request: {:?}", parsed_request);

    println!("Request masuk:\n{}", request);

    // 5️⃣ Body response
    let body = "Hello World!";

    // 6️⃣ Format HTTP response yang valid
    // Penting:
    // - Status line
    // - Header
    // - Baris kosong
    // - Body
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
Content-Type: text/plain\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\
\r\n\
{}",
        body.len(),
        body
    );

    // 7️⃣ Kirim response ke client
    stream.write_all(response.as_bytes())
        .expect("Gagal kirim response");

    // 8️⃣ Flush biar dipaksa keluar dari buffer
    stream.flush().unwrap();

    // Setelah ini koneksi otomatis close karena drop
}