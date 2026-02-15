pub struct Request {
    pub method: String,
    pub path: String,
}

impl Request {
    pub fn parse(buffer: &[u8]) -> Self {
        // This function will parse the HTTP request and return a Request struct.
        let request_str = String::from_utf8_lossy(buffer);
        let request_line = request_str.lines().next().unwrap_or("");

        let parts = request_line.split_whitespace().collect::<Vec<&str>>();
        
        let method = parts.get(0).unwrap_or(&"").to_string();
        let path = parts.get(1).unwrap_or(&"").to_string();
        
        Request { method, path }
    }
}
