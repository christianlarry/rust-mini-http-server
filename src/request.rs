use std::collections::HashMap;
use urlencoding::decode;

pub enum Body{
    Text(String),
    Json(serde_json::Value),
    Form(HashMap<String, String>),
    Empty,
}

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

#[allow(dead_code)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub params: HashMap<String, Vec<String>>,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Body,
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

            let decoded_query_string = decode(query_string).unwrap_or_default().to_string();

            for pair in decoded_query_string.split('&') {
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
        let raw_body = headers.get("Content-Length").and_then(|len| {
            let len: usize = len.parse().ok()?;
            let body_start = request_str.find("\r\n\r\n")? + 4; // Start of body is after the header section
            Some(request_str[body_start..].chars().take(len).collect::<String>())
        });
        let body = parse_body(&headers, raw_body);

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

fn parse_body(headers: &HashMap<String, String>, body_str: Option<String>) -> Body {
    let content_type = headers.get("Content-Type").map(|s| s.as_str()).unwrap_or("");

    match (content_type, body_str) {
        (_, None) => Body::Empty,
        ("text/plain", Some(body)) => Body::Text(body),
        ("application/json", Some(body)) => {
            match serde_json::from_str(&body) {
                Ok(json) => Body::Json(json),
                Err(_) => Body::Empty, // Fallback if JSON parsing fails
            }
        },
        ("application/x-www-form-urlencoded", Some(body)) => {
            let mut form_data = HashMap::new();
            for pair in body.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    form_data.insert(key.to_string(), value.to_string());
                }
            }
            Body::Form(form_data)
        }
        (_, Some(_)) => Body::Text("".to_string()), // Fallback for unsupported content types
    }
}