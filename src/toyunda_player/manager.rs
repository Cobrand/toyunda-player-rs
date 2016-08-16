use iron::{Listening,status};
use iron::prelude::*;
use std::net::ToSocketAddrs;
use std::ops::Drop;
use mount::Mount;
use router::Router;
use staticfile::Static;
use std::sync::{Weak,RwLock};
use ::toyunda_player::state::State as ToyundaState;
use std::ops::Deref;
use serde_json;

pub struct Manager {
    listening:Listening,
    toyunda_state:Weak<RwLock<ToyundaState>>
}

impl Manager {
    fn state_request(toyunda_state:Weak<RwLock<ToyundaState>>) ->  IronResult<Response> {
        use iron::mime::Mime;
        match toyunda_state.upgrade() {
            Some(arc_t) => {
                let json_mime: Mime = "application/json".parse().unwrap();
                let toy_state = arc_t.deref().read().unwrap();
                let json_answer = serde_json::to_string_pretty(&*toy_state).unwrap();
                Ok(Response::with((status::Ok,json_answer,json_mime)))
            },
            None => {
                Ok(Response::with(status::ServiceUnavailable))
            }
        }
    }

    fn command(request:&mut Request) -> IronResult<Response> {
        unimplemented!()
    }

    pub fn new<A : ToSocketAddrs>(address: A, toyunda_state: Weak<RwLock<ToyundaState>>) -> IronResult<Manager> {
        let toyunda_state_cloned = toyunda_state.clone();
        let mut api_handler = Router::new();
        api_handler.get("state",move |request :&mut Request| {
            Self::state_request(toyunda_state_cloned.clone())
        });
        api_handler.get("command",move |request :&mut Request| {
            Self::command(request)
        });
        let mut mount = Mount::new();
        mount.mount("/",Static::new("web/"));
        mount.mount("/api", api_handler);
        let listening =  Iron::new(mount).http(address).unwrap();
        Ok(Manager {
            listening:listening,
            toyunda_state : toyunda_state
        })
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        self.listening.close().unwrap();
    }
}
