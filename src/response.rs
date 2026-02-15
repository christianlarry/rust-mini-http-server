use std::net::TcpStream;
use std::io::Write;

pub struct Response<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> Response<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Response { stream }
    }

    pub fn send(&mut self, body: &str) {
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

        self.stream.write_all(response.as_bytes()).unwrap();
    }
}
