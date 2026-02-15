use std::collections::HashMap;

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    UNKNOWN,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::UNKNOWN => "UNKNOWN",
        }
    }
}

pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub params: HashMap<String, Vec<String>>,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn parse(buffer: &[u8]) -> Self {
        // This function will parse the HTTP request and return a Request struct.
        let request_str = String::from_utf8_lossy(buffer);
        let request_line = request_str.lines().next().unwrap_or("");

        let parts = request_line.split_whitespace().collect::<Vec<&str>>();

        let method = parts.get(0).unwrap_or(&"").to_string();
        let full_path = parts.get(1).unwrap_or(&"").to_string();
        let http_version = parts.get(2).unwrap_or(&"").to_string();

        // Parse query parameters from the path to full path, will be updated if query parameters are found
        let mut query_params: HashMap<String, Vec<String>> = HashMap::new(); // Placeholder for query parameters parsing
        let mut path = full_path.clone();

        if let Some(pos) = full_path.find('?') {
            let query_string = &full_path[pos + 1..];
            path = full_path[..pos].to_string();

            println!("Parsing query string: {}", query_string);

            for pair in query_string.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    query_params.entry(key.to_string()).or_insert_with(Vec::new).push(value.to_string());
                }
            }
        }

        // Parse Headers
        let mut headers = HashMap::new(); // Placeholder for headers parsing
        for line in request_str.lines().skip(1) {
            if line.is_empty() {
                break; // End of headers
            }
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        // Parse Body
        let body = if let Some(length) = headers.get("Content-Length") {
            let len: usize = length.parse().unwrap_or(0);
            
            // Find the start of the body by looking for the double CRLF that separates headers from the body
            let body_start = request_str.find("\r\n\r\n").unwrap_or(0) + 4;

            let body_content = &request_str[body_start..];

            Some(body_content[..len.min(body_content.len())].to_string())
        } else {
            None
        };

        Request {
            method: match method.as_str() {
                "GET" => HttpMethod::GET,
                "POST" => HttpMethod::POST,
                "PUT" => HttpMethod::PUT,
                "DELETE" => HttpMethod::DELETE,
                "HEAD" => HttpMethod::HEAD,
                "OPTIONS" => HttpMethod::OPTIONS,
                "PATCH" => HttpMethod::PATCH,
                _ => HttpMethod::UNKNOWN,
            },
            path: path,
            params: query_params,
            http_version,
            headers,
            body,
        }
    }
}