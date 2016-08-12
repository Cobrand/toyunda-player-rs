use iron::{Listening,status};
use iron::prelude::*;
use std::net::ToSocketAddrs;
use std::ops::Drop;
pub struct Manager {
    listening:Listening
}

impl Manager {
    fn iron_handler(request:&mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }
    pub fn new<A : ToSocketAddrs>(address: A) -> IronResult<Manager> {
        let listening =  Iron::new(Self::iron_handler).http(address).unwrap();
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
