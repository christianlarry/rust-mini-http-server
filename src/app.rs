use crate::router::Router;
use crate::request::{HttpMethod, Request};
use crate::response::Response;
use crate::server::Server;

pub struct App {
    router: Router,
}

impl App {
    pub fn new() -> Self {
        App {
            router: Router::new(),
        }
    }

    pub fn get(&mut self, path: &str, handler: fn(&Request, &mut Response)) {
        self.router.add(&HttpMethod::GET, path, handler);
    }

    pub fn run(self, addr: &str) {
        Server::start(addr, self.router);
    }
}
