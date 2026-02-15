use std::collections::HashMap;
use crate::request::Request;
use crate::response::Response;

pub type Handler = fn(&Request, &mut Response);

pub struct Router {
    routes: HashMap<String, Handler>, // Key: "METHOD:PATH", Value: Handler function
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn add(&mut self, method: &str, path: &str, handler: Handler) {
        let key = format!("{}:{}", method, path);
        self.routes.insert(key, handler);
    }

    pub fn handle(&self, req: &Request, res: &mut Response) {
        let key = format!("{}:{}", req.method, req.path);

        if let Some(handler) = self.routes.get(&key) {
            handler(req, res);
        } else {
            res.send("404 Not Found");
        }
    }
}
