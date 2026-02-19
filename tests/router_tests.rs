use mini_http::request::Method;
use mini_http::router::{RouteGroup, Router};

#[test]
fn test_static_route() {
    let mut router = Router::new();
    router.add(Method::GET, "/hello", |_req, res| res.send("Hello"));

    assert!(router.match_route(&Method::GET, "/hello").is_some());
    assert!(router.match_route(&Method::GET, "/world").is_none());
    assert!(router.match_route(&Method::POST, "/hello").is_none());
}

#[test]
fn test_root_route() {
    let mut router = Router::new();
    router.add(Method::GET, "/", |_req, res| res.send("Root"));

    assert!(router.match_route(&Method::GET, "/").is_some());
    assert!(router.match_route(&Method::GET, "/anything").is_none());
}

#[test]
fn test_single_path_param() {
    let mut router = Router::new();
    router.add(Method::GET, "/users/:id", |_req, res| res.send("User"));

    let (_, params) = router.match_route(&Method::GET, "/users/42").unwrap();
    assert_eq!(params.get("id"), Some(&"42".to_string()));

    let (_, params) = router.match_route(&Method::GET, "/users/alice").unwrap();
    assert_eq!(params.get("id"), Some(&"alice".to_string()));

    assert!(router.match_route(&Method::GET, "/users").is_none());
    assert!(router.match_route(&Method::GET, "/users/42/extra").is_none());
}

#[test]
fn test_multiple_path_params() {
    let mut router = Router::new();
    router.add(Method::GET, "/users/:uid/posts/:pid", |_req, res| {
        res.send("Post")
    });

    let (_, params) = router
        .match_route(&Method::GET, "/users/10/posts/20")
        .unwrap();
    assert_eq!(params.get("uid"), Some(&"10".to_string()));
    assert_eq!(params.get("pid"), Some(&"20".to_string()));
}

#[test]
fn test_wildcard_route() {
    let mut router = Router::new();
    router.add(Method::GET, "/files/*path", |_req, res| res.send("File"));

    let (_, params) = router
        .match_route(&Method::GET, "/files/docs/readme.md")
        .unwrap();
    assert_eq!(params.get("path"), Some(&"docs/readme.md".to_string()));

    let (_, params) = router
        .match_route(&Method::GET, "/files/image.png")
        .unwrap();
    assert_eq!(params.get("path"), Some(&"image.png".to_string()));
}

#[test]
fn test_method_matching() {
    let mut router = Router::new();
    router.add(Method::GET, "/resource", |_req, res| res.send("GET"));
    router.add(Method::POST, "/resource", |_req, res| res.send("POST"));
    router.add(Method::PUT, "/resource", |_req, res| res.send("PUT"));
    router.add(Method::DELETE, "/resource", |_req, res| res.send("DELETE"));

    assert!(router.match_route(&Method::GET, "/resource").is_some());
    assert!(router.match_route(&Method::POST, "/resource").is_some());
    assert!(router.match_route(&Method::PUT, "/resource").is_some());
    assert!(router.match_route(&Method::DELETE, "/resource").is_some());
    assert!(router.match_route(&Method::PATCH, "/resource").is_none());
}

#[test]
fn test_path_exists_for_405() {
    let mut router = Router::new();
    router.add(Method::GET, "/only-get", |_req, res| res.send("OK"));

    assert!(router.path_exists("/only-get"));
    assert!(!router.path_exists("/not-here"));
    assert!(router.match_route(&Method::POST, "/only-get").is_none());
}

#[test]
fn test_route_group() {
    let mut router = Router::new();

    let mut api = RouteGroup::new("/api/v1");
    api.get("/users", |_req, res| res.send("Users"));
    api.post("/users", |_req, res| res.send("Create"));
    api.get("/users/:id", |_req, res| res.send("User"));

    router.add_group(api);

    assert!(router.match_route(&Method::GET, "/api/v1/users").is_some());
    assert!(router.match_route(&Method::POST, "/api/v1/users").is_some());

    let (_, params) = router
        .match_route(&Method::GET, "/api/v1/users/42")
        .unwrap();
    assert_eq!(params.get("id"), Some(&"42".to_string()));

    // Should NOT match without prefix
    assert!(router.match_route(&Method::GET, "/users").is_none());
}

#[test]
fn test_route_priority_first_match_wins() {
    let mut router = Router::new();
    router.add(Method::GET, "/users/me", |_req, res| res.send("Me"));
    router.add(Method::GET, "/users/:id", |_req, res| res.send("User"));

    // Static route registered first should match first
    let (handler, params) = router.match_route(&Method::GET, "/users/me").unwrap();
    assert!(params.is_empty()); // Static match, no params

    // Dynamic route matches other values
    let (_, params) = router.match_route(&Method::GET, "/users/42").unwrap();
    assert_eq!(params.get("id"), Some(&"42".to_string()));

    let _ = handler; // suppress unused warning
}

#[test]
fn test_nested_route_groups() {
    let mut router = Router::new();

    let mut v1 = RouteGroup::new("/api/v1");
    v1.get("/health", |_req, res| res.send("ok"));

    let mut v2 = RouteGroup::new("/api/v2");
    v2.get("/health", |_req, res| res.send("ok"));

    router.add_group(v1);
    router.add_group(v2);

    assert!(router.match_route(&Method::GET, "/api/v1/health").is_some());
    assert!(router.match_route(&Method::GET, "/api/v2/health").is_some());
}
