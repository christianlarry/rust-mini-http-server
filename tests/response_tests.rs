use mini_http::cookie::{CookieBuilder, SameSite};
use mini_http::response::Response;

#[test]
fn test_text_response() {
    let mut res = Response::new();
    res.send("Hello, World!");

    assert_eq!(res.status_code, 200);
    assert_eq!(res.body, b"Hello, World!");
    assert!(res
        .headers
        .get("content-type")
        .unwrap()
        .contains("text/plain"));
    assert!(res.sent);
}

#[test]
fn test_json_response() {
    let mut res = Response::new();
    res.json(&serde_json::json!({"status": "ok", "count": 42}));

    assert_eq!(res.status_code, 200);
    assert!(res
        .headers
        .get("content-type")
        .unwrap()
        .contains("application/json"));

    let body: serde_json::Value = serde_json::from_slice(&res.body).unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["count"], 42);
}

#[test]
fn test_html_response() {
    let mut res = Response::new();
    res.html("<h1>Hello</h1>");

    assert!(res
        .headers
        .get("content-type")
        .unwrap()
        .contains("text/html"));
    assert_eq!(res.body, b"<h1>Hello</h1>");
}

#[test]
fn test_status_code_chaining() {
    let mut res = Response::new();
    res.status(201).send("Created");

    assert_eq!(res.status_code, 201);
    assert_eq!(res.body, b"Created");
}

#[test]
fn test_custom_headers() {
    let mut res = Response::new();
    res.header("X-Custom", "value123")
        .header("X-Another", "abc")
        .send("OK");

    assert_eq!(res.headers.get("X-Custom").unwrap(), "value123");
    assert_eq!(res.headers.get("X-Another").unwrap(), "abc");
}

#[test]
fn test_redirect() {
    let mut res = Response::new();
    res.redirect("/new-location");

    assert_eq!(res.status_code, 302);
    assert_eq!(res.headers.get("location").unwrap(), "/new-location");
}

#[test]
fn test_permanent_redirect() {
    let mut res = Response::new();
    res.status(301).redirect("/permanent");

    assert_eq!(res.status_code, 301);
    assert_eq!(res.headers.get("location").unwrap(), "/permanent");
}

#[test]
fn test_send_status_only() {
    let mut res = Response::new();
    res.send_status(204);

    assert_eq!(res.status_code, 204);
    assert!(res.body.is_empty());
}

#[test]
fn test_set_cookie() {
    let mut res = Response::new();
    res.set_cookie(CookieBuilder::new("session", "abc123").path("/").http_only());

    assert_eq!(res.set_cookies.len(), 1);
    let cookie = &res.set_cookies[0];
    assert!(cookie.contains("session=abc123"));
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("HttpOnly"));
}

#[test]
fn test_remove_cookie() {
    let mut res = Response::new();
    res.remove_cookie("session");

    assert_eq!(res.set_cookies.len(), 1);
    let cookie = &res.set_cookies[0];
    assert!(cookie.contains("session="));
    assert!(cookie.contains("Max-Age=0"));
}

#[test]
fn test_to_bytes_format() {
    let mut res = Response::new();
    res.status(200).send("OK");

    let bytes = res.to_bytes();
    let text = String::from_utf8_lossy(&bytes);

    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("Content-Length: 2\r\n"));
    assert!(text.contains("\r\n\r\nOK"));
}

#[test]
fn test_cookie_builder() {
    let cookie = CookieBuilder::new("token", "xyz")
        .path("/api")
        .domain("example.com")
        .max_age(3600)
        .secure()
        .http_only()
        .same_site(SameSite::Strict)
        .build();

    assert!(cookie.contains("token=xyz"));
    assert!(cookie.contains("Path=/api"));
    assert!(cookie.contains("Domain=example.com"));
    assert!(cookie.contains("Max-Age=3600"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
}

#[test]
fn test_to_bytes_includes_cookies() {
    let mut res = Response::new();
    res.set_cookie(CookieBuilder::new("a", "1"));
    res.set_cookie(CookieBuilder::new("b", "2"));
    res.send("OK");

    let bytes = res.to_bytes();
    let text = String::from_utf8_lossy(&bytes);

    assert!(text.contains("Set-Cookie: a=1\r\n"));
    assert!(text.contains("Set-Cookie: b=2\r\n"));
}
