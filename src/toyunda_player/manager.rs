use iron::{Listening, status};
use iron::prelude::*;
use std::net::ToSocketAddrs;
use std::ops::Drop;
use mount::Mount;
use router::Router;
use staticfile::Static;
use std::sync::{Weak, RwLock, Arc, Mutex};
use ::toyunda_player::state::State as ToyundaState;
use ::toyunda_player::command::*;
use ::toyunda_player::yaml_meta::*;
use ::toyunda_player::video_meta::*;

use ::utils::for_each_in_dir;
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::sync::mpsc::{channel, Sender, Receiver};
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
    #[serde(rename = "quit")]
    Quit,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "quit_on_finish")]
    QuitOnFinish,
    #[serde(rename = "toggle_subtitles")]
    ToggleSubtitles,
}

#[derive(Debug,Deserialize)]
struct WebCommand {
    command: WebCommandType,
    id: Option<u32>,
}

pub struct Manager {
    listening: Listening,
    toyunda_state: Weak<RwLock<ToyundaState>>,
    pub receiver: Receiver<Command>,
    _yaml_files: Arc<Vec<YamlMeta>>,
}

impl Manager {
    fn log(&self, message: &str) {
        match self.toyunda_state.upgrade() {
            Some(toy_state_arc) => {
                toy_state_arc.write().unwrap().logs.push(String::from(message));
            }
            None => {}
        }
    }

    fn add_yaml_file<P: AsRef<Path>>(yaml_files: &mut Vec<YamlMeta>,
                                     file: P)
                                     -> Result<(), String> {
        let file = file.as_ref();

        match VideoMeta::from_yaml(file) {
            Ok(video_meta) => {
                yaml_files.push(YamlMeta {
                    yaml_path: PathBuf::from(file),
                    video_meta: video_meta.fix_paths(file),
                });
                Ok(())
            }
            Err(e) => Err(format!("Error when loading yaml file '{}' : {}", file.display(), e)),
        }
    }

    fn parse_yaml_directory<P: AsRef<Path>>(directory: P) -> Result<Vec<YamlMeta>, String> {
        let mut yaml_files: Vec<YamlMeta> = Vec::new();
        let (paths, errs) = for_each_in_dir(directory,
                                            3,
                                            &|path| {
                                                match path.extension() {
                                                    Some(s) if s == "yaml" => true,
                                                    _ => false,
                                                }
                                            });
        for err in errs {
            let _tmp_str = format!("IoError '{}' when parsing yaml dir", err);
            error!("{}", _tmp_str);
            // self.log(_tmp_str.as_str());
        }
        for path in paths {
            match Self::add_yaml_file(&mut yaml_files, &path) {
                Ok(()) => {}
                Err(err_string) => {
                    error!("{}", err_string);
                    // self.log(err_string.as_str());
                }
            }
        }
        Ok(yaml_files)
    }

    fn state_request(toyunda_state: Weak<RwLock<ToyundaState>>) -> IronResult<Response> {
        use iron::mime::Mime;
        match toyunda_state.upgrade() {
            Some(arc_t) => {
                let json_mime: Mime = "application/json".parse().unwrap();
                let toy_state = arc_t.deref().read().unwrap();
                let json_answer = serde_json::to_string_pretty(&*toy_state).unwrap();
                Ok(Response::with((status::Ok, json_answer, json_mime)))
            }
            None => Ok(Response::with(status::ServiceUnavailable)),
        }
    }

    fn list_request(list: Weak<Vec<YamlMeta>>) -> IronResult<Response> {
        use iron::mime::Mime;
        let json_mime: Mime = "application/json".parse().unwrap();
        match list.upgrade() {
            Some(list) => {
                let json_answer = serde_json::to_string_pretty(list.as_ref()).unwrap();
                Ok(Response::with((status::Ok, json_answer, json_mime)))
            }
            None => Ok(Response::with((status::Gone, "{}", json_mime))),
        }
    }

    fn command(request: &mut Request,
               tx: Sender<Command>,
               list: Weak<Vec<YamlMeta>>)
               -> IronResult<Response> {
        let web_command = request.get_ref::<bodyparser::Struct<WebCommand>>();
        match web_command {
            Ok(&Some(ref web_command)) => {
                let command: Result<Command, String> = match web_command.command {
                    WebCommandType::PlayNext => Ok(Command::PlayNext),
                    WebCommandType::ClearQueue => Ok(Command::ClearQueue),
                    WebCommandType::Stop => Ok(Command::Stop),
                    WebCommandType::Pause => Ok(Command::TogglePause),
                    WebCommandType::AddToQueue => {
                        if let Some(id) = web_command.id {
                            if let Some(list) = list.upgrade() {
                                if let Some(ref yaml_meta) = list.get(id as usize) {
                                    Ok(Command::AddToQueue(yaml_meta.video_meta.clone()))
                                } else {
                                    Err(format!("Bad Index {}", id))
                                }
                            } else {
                                Err(String::from("Gone"))
                            }
                        } else {
                            Err(String::from("'id' field is needed"))
                        }
                    }
                    _ => unimplemented!(),
                };
                println!("command : {:?}", command);
                if let Ok(command) = command {
                    if let Err(e) = tx.send(command) {
                        error!("An error happened when trying\
                        to send a command to the other thread : {}",e);
                        // TODO make it Err(_) intstead
                        Ok(Response::with(status::InternalServerError))
                    } else {
                        Ok(Response::with(status::Ok))
                    }
                } else {
                    Ok(Response::with(status::BadRequest))
                }
            }
            Ok(&None) => Ok(Response::with(status::BadRequest)),
            Err(err) => Err(IronError::new(err, status::BadRequest)),
        }
    }

    pub fn new<A: ToSocketAddrs>(address: A,
                                 toyunda_state: Weak<RwLock<ToyundaState>>,
                                 yaml_directories: Vec<PathBuf>)
                                 -> IronResult<Manager> {
        let mut yaml_files: Vec<YamlMeta> = Vec::new();
        for dir in yaml_directories {
            if yaml_files.is_empty() {
                yaml_files = Self::parse_yaml_directory(&dir).unwrap();
            } else {
                yaml_files.extend(Self::parse_yaml_directory(&dir).unwrap());
            }
        }
        let yaml_files = Arc::new(yaml_files);
        let (tx, rx) = channel();
        let toyunda_state_cloned = toyunda_state.clone();
        let mut api_handler = Router::new();
        api_handler.get("state", move |_r: &mut Request| {
            Self::state_request(toyunda_state_cloned.clone())
        });
        let tx_command = Mutex::new(tx);
        let weak_list = Arc::downgrade(&yaml_files);
        let weak_list2 = weak_list.clone();
        api_handler.post("command", move |request: &mut Request| {
            let tx_command = tx_command.lock().unwrap().clone();
            Self::command(request, tx_command, weak_list2.clone())
        });
        api_handler.get("listing", move |_r: &mut Request| {
            Self::list_request(weak_list.clone())
        });
        let mut mount = Mount::new();
        mount.mount("/", Static::new("web/"));
        mount.mount("/api", api_handler);
        let listening = Iron::new(mount).http(address).unwrap();
        Ok(Manager {
            listening: listening,
            toyunda_state: toyunda_state,
            _yaml_files: yaml_files,
            receiver: rx,
        })
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        self.listening.close().unwrap();
    }
}
