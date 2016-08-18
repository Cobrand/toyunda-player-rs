use iron::{Listening,status};
use iron::prelude::*;
use std::net::ToSocketAddrs;
use std::ops::Drop;
use mount::Mount;
use router::Router;
use staticfile::Static;
use std::sync::{Weak,RwLock,Arc,Mutex};
use ::toyunda_player::state::State as ToyundaState;
use ::toyunda_player::command::*;
use std::ops::Deref;
use std::sync::mpsc::Sender;
use serde_json;
use bodyparser;

#[derive(Debug,Deserialize,Clone)]
enum WebCommandType {
    #[serde(rename = "play_next")]
    PlayNext,
    #[serde(rename = "add_to_queue")]
    AddToQueue,
    #[serde(rename = "clear_queue")]
    ClearQueue,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "unpause")]
    Unpause,
    #[serde(rename = "quit")]
    Quit,
    #[serde(rename = "quit_on_finish")]
    QuitOnFinish,
    #[serde(rename = "toggle_subtitles")]
    ToggleSubtitles,
}

#[derive(Debug,Deserialize)]
struct WebCommand {
    command:WebCommandType,
    music_name:Option<String>,
    variant:Option<u32>
}

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

    fn command(request:&mut Request, tx : Sender<Command> ) -> IronResult<Response> {
        let web_command = request.get_ref::<bodyparser::Struct<WebCommand>>();
        match web_command {
            Ok(&Some(ref web_command)) => {
                let command : Result<Command,String> = match web_command.command {
                    WebCommandType::PlayNext => Ok(Command::PlayNext),
                    WebCommandType::ClearQueue => Ok(Command::ClearQueue),
                    _ => unimplemented!()
                };
                println!("command : {:?}",command);
                if let Ok(command) = command {
                    tx.send(command);
                    Ok(Response::with(status::Ok))
                } else {
                    Ok(Response::with(status::BadRequest))
                }
            },
            Ok(&None) => {
                Ok(Response::with(status::BadRequest))
            },
            Err(err) => {
                Err(IronError::new(err,status::BadRequest))
            }
        }
    }

    pub fn new<A : ToSocketAddrs>(address: A,
                                  toyunda_state: Weak<RwLock<ToyundaState>>,
                                  tx_command : Sender<Command>) -> IronResult<Manager> {
        let toyunda_state_cloned = toyunda_state.clone();
        let mut api_handler = Router::new();
        api_handler.get("state",move |request :&mut Request| {
            Self::state_request(toyunda_state_cloned.clone())
        });
        let tx_command = Mutex::new(tx_command);
        api_handler.post("command",move |request :&mut Request| {
            let tx_command = tx_command.lock().unwrap().clone();
            Self::command(request,tx_command)
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
