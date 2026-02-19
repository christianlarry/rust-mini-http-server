//! HTTP router with path parameter support and route groups.
//!
//! Provides pattern-based route matching with support for:
//! - Static routes (`/users`)
//! - Dynamic path parameters (`:id` matches a single segment)
//! - Wildcard routes (`*path` matches everything)
//! - Route groups with shared prefix and middleware
//! - All standard HTTP methods

use std::collections::HashMap;

use regex::Regex;

use crate::middleware::Middleware;
use crate::request::{Method, Request};
use crate::response::Response;

/// Handler function type — supports both function pointers and closures.
pub type Handler = Box<dyn Fn(&mut Request, &mut Response) + Send + Sync + 'static>;

/// A single route definition with compiled regex pattern.
pub struct Route {
    /// HTTP method for this route.
    pub method: Method,
    /// Original pattern string (e.g., `/users/:id`).
    pub pattern: String,
    /// Compiled regex for matching.
    regex: Regex,
    /// Ordered list of parameter names extracted from the pattern.
    param_names: Vec<String>,
    /// The handler function.
    handler: Handler,
}

impl Route {
    /// Create a new route from a method, pattern, and handler.
    fn new(method: Method, pattern: &str, handler: Handler) -> Self {
        let (regex, param_names) = compile_pattern(pattern);
        Route {
            method,
            pattern: pattern.to_string(),
            regex,
            param_names,
            handler,
        }
    }

    /// Try to match a request path. Returns extracted params if matched.
    fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        self.regex.captures(path).map(|caps| {
            let mut params = HashMap::new();
            for (i, name) in self.param_names.iter().enumerate() {
                if let Some(m) = caps.get(i + 1) {
                    params.insert(name.clone(), m.as_str().to_string());
                }
            }
            params
        })
    }
}

impl std::fmt::Debug for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("pattern", &self.pattern)
            .field("param_names", &self.param_names)
            .finish()
    }
}

/// Compile a route pattern into a regex and extract parameter names.
///
/// - `:name` matches a single path segment: `([^/]+)`
/// - `*name` matches everything remaining: `(.*)`
/// - Other segments are escaped for literal matching.
fn compile_pattern(pattern: &str) -> (Regex, Vec<String>) {
    let mut regex_str = String::from("^");
    let mut param_names = Vec::new();

    if pattern == "/" {
        regex_str.push_str("/$");
        return (Regex::new(&regex_str).unwrap(), param_names);
    }

    for segment in pattern.split('/') {
        if segment.is_empty() {
            continue;
        }
        regex_str.push('/');
        if let Some(name) = segment.strip_prefix(':') {
            param_names.push(name.to_string());
            regex_str.push_str("([^/]+)");
        } else if let Some(name) = segment.strip_prefix('*') {
            param_names.push(name.to_string());
            regex_str.push_str("(.+)");
        } else {
            regex_str.push_str(&regex::escape(segment));
        }
    }

    regex_str.push('$');
    (Regex::new(&regex_str).unwrap(), param_names)
}

/// The main HTTP router.
///
/// Routes are matched in registration order. The first matching route wins.
pub struct Router {
    routes: Vec<Route>,
    /// WebSocket route handlers (method is always GET with Upgrade header).
    ws_routes: Vec<WsRoute>,
}

/// A WebSocket route definition.
pub struct WsRoute {
    /// URL pattern.
    pub pattern: String,
    /// Compiled regex.
    regex: Regex,
    /// Parameter names.
    param_names: Vec<String>,
    /// WebSocket handler.
    pub handler: WsHandler,
}

/// WebSocket handler type.
pub type WsHandler = Box<dyn Fn(crate::websocket::WebSocket) + Send + Sync + 'static>;

impl WsRoute {
    fn new(pattern: &str, handler: WsHandler) -> Self {
        let (regex, param_names) = compile_pattern(pattern);
        WsRoute {
            pattern: pattern.to_string(),
            regex,
            param_names,
            handler,
        }
    }

    fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        self.regex.captures(path).map(|caps| {
            let mut params = HashMap::new();
            for (i, name) in self.param_names.iter().enumerate() {
                if let Some(m) = caps.get(i + 1) {
                    params.insert(name.clone(), m.as_str().to_string());
                }
            }
            params
        })
    }
}

impl Router {
    /// Create a new empty router.
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            ws_routes: Vec::new(),
        }
    }

    /// Add a route with the given method, pattern, and handler.
    pub fn add<F>(&mut self, method: Method, pattern: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .push(Route::new(method, pattern, Box::new(handler)));
    }

    /// Add a WebSocket route.
    pub fn add_ws<F>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(crate::websocket::WebSocket) + Send + Sync + 'static,
    {
        self.ws_routes
            .push(WsRoute::new(pattern, Box::new(handler)));
    }

    /// Register a route group, prepending its prefix to all routes.
    pub fn add_group(&mut self, group: RouteGroup) {
        for (method, pattern, handler) in group.routes {
            let full_pattern = format!("{}{}", group.prefix, pattern);
            self.routes
                .push(Route::new(method, &full_pattern, handler));
        }
    }

    /// Match a request to a route. Returns the handler and extracted path params.
    pub fn match_route(
        &self,
        method: &Method,
        path: &str,
    ) -> Option<(&Handler, HashMap<String, String>)> {
        for route in &self.routes {
            if route.method == *method {
                if let Some(params) = route.match_path(path) {
                    return Some((&route.handler, params));
                }
            }
        }
        None
    }

    /// Match a WebSocket route.
    pub fn match_ws_route(&self, path: &str) -> Option<(&WsHandler, HashMap<String, String>)> {
        for route in &self.ws_routes {
            if let Some(params) = route.match_path(path) {
                return Some((&route.handler, params));
            }
        }
        None
    }

    /// Check if any route matches the path (regardless of method).
    /// Used for 405 Method Not Allowed detection.
    pub fn path_exists(&self, path: &str) -> bool {
        self.routes.iter().any(|route| route.match_path(path).is_some())
    }

    /// Handle a request: match route, inject params, call handler.
    /// Returns true if a route was found, false otherwise.
    pub fn handle(&self, req: &mut Request, res: &mut Response) -> bool {
        if let Some((handler, params)) = self.match_route(&req.method, &req.path) {
            req.params = params;
            handler(req, res);
            true
        } else if self.path_exists(&req.path) {
            // Path exists but method doesn't match
            res.status(405).send("Method Not Allowed");
            true
        } else {
            false
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// A group of routes sharing a common prefix and middleware.
///
/// # Example
/// ```
/// use mini_http::router::RouteGroup;
///
/// let mut api = RouteGroup::new("/api/v1");
/// api.get("/users", |_req, res| {
///     res.json(&serde_json::json!({"users": []}));
/// });
/// api.post("/users", |_req, res| {
///     res.status(201).send("Created");
/// });
/// ```
pub struct RouteGroup {
    /// URL prefix for all routes in this group.
    pub prefix: String,
    /// Routes in this group (method, pattern, handler).
    pub routes: Vec<(Method, String, Handler)>,
    /// Middleware specific to this group.
    pub middleware: Vec<Box<dyn Middleware>>,
}

impl RouteGroup {
    /// Create a new route group with the given prefix.
    pub fn new(prefix: &str) -> Self {
        RouteGroup {
            prefix: prefix.trim_end_matches('/').to_string(),
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Add a GET route to this group.
    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .push((Method::GET, path.to_string(), Box::new(handler)));
    }

    /// Add a POST route to this group.
    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .push((Method::POST, path.to_string(), Box::new(handler)));
    }

    /// Add a PUT route to this group.
    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .push((Method::PUT, path.to_string(), Box::new(handler)));
    }

    /// Add a DELETE route to this group.
    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .push((Method::DELETE, path.to_string(), Box::new(handler)));
    }

    /// Add a PATCH route to this group.
    pub fn patch<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&mut Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .push((Method::PATCH, path.to_string(), Box::new(handler)));
    }

    /// Add middleware to this route group.
    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middleware.push(Box::new(middleware));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_route_matching() {
        let mut router = Router::new();
        router.add(Method::GET, "/hello", |_req, res| {
            res.send("Hello!");
        });

        let (_, params) = router.match_route(&Method::GET, "/hello").unwrap();
        assert!(params.is_empty());
        assert!(router.match_route(&Method::GET, "/world").is_none());
    }

    #[test]
    fn test_path_param_matching() {
        let mut router = Router::new();
        router.add(Method::GET, "/users/:id", |_req, res| {
            res.send("User");
        });

        let (_, params) = router.match_route(&Method::GET, "/users/123").unwrap();
        assert_eq!(params.get("id").unwrap(), "123");
        assert!(router.match_route(&Method::GET, "/users").is_none());
    }

    #[test]
    fn test_multiple_path_params() {
        let mut router = Router::new();
        router.add(Method::GET, "/users/:user_id/posts/:post_id", |_req, res| {
            res.send("Post");
        });

        let (_, params) = router
            .match_route(&Method::GET, "/users/42/posts/7")
            .unwrap();
        assert_eq!(params.get("user_id").unwrap(), "42");
        assert_eq!(params.get("post_id").unwrap(), "7");
    }

    #[test]
    fn test_wildcard_route() {
        let mut router = Router::new();
        router.add(Method::GET, "/files/*path", |_req, res| {
            res.send("File");
        });

        let (_, params) = router
            .match_route(&Method::GET, "/files/docs/readme.md")
            .unwrap();
        assert_eq!(params.get("path").unwrap(), "docs/readme.md");
    }

    #[test]
    fn test_method_not_allowed() {
        let mut router = Router::new();
        router.add(Method::GET, "/resource", |_req, res| {
            res.send("OK");
        });

        assert!(router.match_route(&Method::POST, "/resource").is_none());
        assert!(router.path_exists("/resource"));
    }

    #[test]
    fn test_route_group() {
        let mut router = Router::new();
        let mut group = RouteGroup::new("/api/v1");
        group.get("/users", |_req, res| {
            res.send("Users");
        });
        router.add_group(group);

        assert!(router.match_route(&Method::GET, "/api/v1/users").is_some());
        assert!(router.match_route(&Method::GET, "/users").is_none());
    }

    #[test]
    fn test_root_route() {
        let mut router = Router::new();
        router.add(Method::GET, "/", |_req, res| {
            res.send("Root");
        });

        assert!(router.match_route(&Method::GET, "/").is_some());
    }
}
