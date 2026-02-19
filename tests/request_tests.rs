use mini_http::request::{Body, Method, Request};

#[test]
fn test_parse_simple_get() {
    let raw = b"GET / HTTP/1.1\r\nHost: localhost:8080\r\n\r\n";
    let req = Request::parse(raw).unwrap();

    assert_eq!(req.method, Method::GET);
    assert_eq!(req.path, "/");
    assert_eq!(req.http_version, "HTTP/1.1");
    assert_eq!(req.header("host"), Some("localhost:8080"));
}

#[test]
fn test_parse_get_with_query_params() {
    let raw = b"GET /search?q=rust&page=2&tag=web&tag=http HTTP/1.1\r\n\r\n";
    let req = Request::parse(raw).unwrap();

    assert_eq!(req.path, "/search");
    assert_eq!(req.query_param("q"), Some("rust"));
    assert_eq!(req.query_param("page"), Some("2"));

    let tags = req.query_params("tag").unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"web".to_string()));
    assert!(tags.contains(&"http".to_string()));
}

#[test]
fn test_parse_post_with_json_body() {
    let body = r#"{"name":"Alice","age":30}"#;
    let raw = format!(
        "POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(), body
    );
    let req = Request::parse(raw.as_bytes()).unwrap();

    assert_eq!(req.method, Method::POST);
    assert_eq!(req.path, "/api/users");

    match &req.body {
        Body::Json(v) => {
            assert_eq!(v["name"], "Alice");
            assert_eq!(v["age"], 30);
        }
        _ => panic!("Expected JSON body"),
    }

    // Test the typed deserialization
    #[derive(serde::Deserialize)]
    struct User {
        name: String,
        age: u32,
    }
    let user: User = req.json().unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
}

#[test]
fn test_parse_post_with_form_body() {
    let body = "username=alice&password=s3cret&remember=on";
    let raw = format!(
        "POST /login HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{}",
        body.len(), body
    );
    let req = Request::parse(raw.as_bytes()).unwrap();

    assert_eq!(req.form_field("username"), Some("alice"));
    assert_eq!(req.form_field("password"), Some("s3cret"));
    assert_eq!(req.form_field("remember"), Some("on"));
}

#[test]
fn test_parse_cookies() {
    let raw = b"GET / HTTP/1.1\r\nCookie: session_id=abc123; theme=dark; lang=en\r\n\r\n";
    let req = Request::parse(raw).unwrap();

    assert_eq!(req.cookie("session_id"), Some("abc123"));
    assert_eq!(req.cookie("theme"), Some("dark"));
    assert_eq!(req.cookie("lang"), Some("en"));
    assert_eq!(req.cookie("nonexistent"), None);
}

#[test]
fn test_parse_all_methods() {
    for (method_str, expected) in [
        ("GET", Method::GET),
        ("POST", Method::POST),
        ("PUT", Method::PUT),
        ("DELETE", Method::DELETE),
        ("PATCH", Method::PATCH),
        ("HEAD", Method::HEAD),
        ("OPTIONS", Method::OPTIONS),
    ] {
        let raw = format!("{} /test HTTP/1.1\r\n\r\n", method_str);
        let req = Request::parse(raw.as_bytes()).unwrap();
        assert_eq!(req.method, expected, "Failed for {}", method_str);
    }
}

#[test]
fn test_content_type_detection() {
    let raw = b"GET / HTTP/1.1\r\nContent-Type: application/json\r\n\r\n";
    let req = Request::parse(raw).unwrap();
    assert_eq!(req.content_type(), Some("application/json"));
}

#[test]
fn test_accepts_header() {
    let raw = b"GET / HTTP/1.1\r\nAccept: text/html, application/json\r\n\r\n";
    let req = Request::parse(raw).unwrap();

    assert!(req.accepts("text/html"));
    assert!(req.accepts("application/json"));
    assert!(!req.accepts("text/xml"));
}

#[test]
fn test_websocket_upgrade_detection() {
    let raw = b"GET /ws HTTP/1.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
    let req = Request::parse(raw).unwrap();
    assert!(req.is_websocket_upgrade());

    let raw = b"GET / HTTP/1.1\r\n\r\n";
    let req = Request::parse(raw).unwrap();
    assert!(!req.is_websocket_upgrade());
}

#[test]
fn test_url_encoded_query_params() {
    let raw = b"GET /search?q=hello%20world&tag=rust%26web HTTP/1.1\r\n\r\n";
    let req = Request::parse(raw).unwrap();

    assert_eq!(req.query_param("q"), Some("hello world"));
    assert_eq!(req.query_param("tag"), Some("rust&web"));
}

#[test]
fn test_empty_body() {
    let raw = b"GET / HTTP/1.1\r\n\r\n";
    let req = Request::parse(raw).unwrap();
    assert!(matches!(req.body, Body::Empty));
}

#[test]
fn test_extensions() {
    let mut req = Request::parse(b"GET / HTTP/1.1\r\n\r\n").unwrap();

    req.set_extension("user_id", 42u64);
    assert_eq!(req.get_extension::<u64>("user_id"), Some(&42));

    req.set_extension("name", "Alice".to_string());
    assert_eq!(
        req.get_extension::<String>("name"),
        Some(&"Alice".to_string())
    );
}
