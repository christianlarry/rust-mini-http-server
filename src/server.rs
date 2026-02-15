use std::net::TcpListener;
use std::io::Read;

use crate::router::Router;
use crate::request::Request;
use crate::response::Response;

pub struct Server;

impl Server {
    pub fn start(addr: &str, router: Router) {
        let listener = TcpListener::bind(addr).unwrap();
        println!("Server running at http://{}", addr);

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap();

            let req = Request::parse(&buffer[..bytes_read]);

            let mut res = Response::new(&mut stream);

            router.handle(&req, &mut res);
        }
    }
}
