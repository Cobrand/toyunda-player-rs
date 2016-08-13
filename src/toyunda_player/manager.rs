use iron::{Listening,status};
use iron::prelude::*;
use std::net::ToSocketAddrs;
use std::ops::Drop;
use mount::Mount;
use staticfile::Static;

pub struct Manager {
    listening:Listening
}

impl Manager {
    fn api_handler(request:&mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }
    pub fn new<A : ToSocketAddrs>(address: A) -> IronResult<Manager> {
        let mut mount = Mount::new();
        mount.mount("/",Static::new("web/"));
        mount.mount("/api", Self::api_handler);
        let listening =  Iron::new(mount).http(address).unwrap();
        Ok(Manager {
            listening:listening
        })
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        self.listening.close().unwrap();
    }
}
