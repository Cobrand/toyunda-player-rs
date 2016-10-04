use iron::{Listening, status};
use iron::prelude::*;
use std::net::ToSocketAddrs;
use std::ops::Drop;
use mount::Mount;
use router::Router;
use staticfile::Static;
use std::sync::{Weak, RwLock, Arc, Mutex};
use super::state::State as ToyundaState;
use super::command::*;
use super::video_meta::*;
use iron::mime::Mime;

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
    #[serde(rename = "add_multiple_to_queue")]
    AddMultipleToQueue,
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
    #[serde(rename = "pause_before_next")]
    PauseBeforeNext,
    #[serde(rename = "toggle_subtitles")]
    ToggleSubtitles,
    #[serde(rename = "announcement")]
    Announcement,
}

#[derive(Debug,Deserialize)]
struct WebCommand {
    command: WebCommandType,
    id: Option<u32>,
    list: Option<Vec<u32>>,
    text: Option<String>
}

pub struct Manager {
    listening: Listening,
    pub receiver: Receiver<Command>,
    _yaml_files: Arc<Vec<VideoMeta>>,
}

impl Manager {
    fn add_yaml_file<P: AsRef<Path>>(yaml_files: &mut Vec<VideoMeta>,
                                     file: P)
                                     -> Result<(), String> {
        let file = file.as_ref();

        match VideoMeta::from_yaml(file) {
            Ok(video_meta) => {
                yaml_files.push(video_meta.fix_paths(file));
                Ok(())
            }
            Err(e) => Err(format!("Error when loading yaml file '{}' : {}", file.display(), e)),
        }
    }

    fn parse_yaml_directory<P: AsRef<Path>>(directory: P) -> Result<Vec<VideoMeta>, String> {
        let mut yaml_files: Vec<VideoMeta> = Vec::new();
        let (paths, errs) = for_each_in_dir(directory,
                                            3,
                                            &|path| {
                                                match path.extension() {
                                                    Some(s) if s == "yaml" => true,
                                                    _ => false,
                                                }
                                            });
        for err in errs {
            error!("IoError '{}' when parsing yaml dir", err);
        }
        for path in paths {
            match Self::add_yaml_file(&mut yaml_files, &path) {
                Ok(()) => {}
                Err(err_string) => {
                    error!("{}", err_string);
                }
            }
        }
        Ok(yaml_files)
    }

    fn state_request(toyunda_state: Weak<RwLock<ToyundaState>>) -> IronResult<Response> {
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

    fn list_request(list: Weak<Vec<VideoMeta>>) -> IronResult<Response> {
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
               list: Weak<Vec<VideoMeta>>)
               -> IronResult<Response> {
        let web_command = request.get_ref::<bodyparser::Struct<WebCommand>>();
        match web_command {
            Ok(&Some(ref web_command)) => {
                let commands: Result<Vec<Command>, String> = match web_command.command {
                    WebCommandType::PlayNext => Ok(vec![Command::PlayNext]),
                    WebCommandType::ClearQueue => Ok(vec![Command::ClearQueue]),
                    WebCommandType::Stop => Ok(vec![Command::Stop]),
                    WebCommandType::Pause => Ok(vec![Command::TogglePause]),
                    WebCommandType::AddToQueue => {
                        if let Some(id) = web_command.id {
                            if let Some(list) = list.upgrade() {
                                if let Some(video_meta) = list.get(id as usize) {
                                    Ok(vec![Command::AddToQueue(video_meta.clone())])
                                } else {
                                    Err(format!("Bad Index {}", id))
                                }
                            } else {
                                Err(String::from("Gone"))
                            }
                        } else {
                            Err(String::from("'id' field is needed"))
                        }
                    },
                    WebCommandType::AddMultipleToQueue => {
                        if let Some(ref ids) = web_command.list {
                            if let Some(list) = list.upgrade() {
                                let commands = ids.iter().filter_map(|id|{
                                    list.get(*id as usize)
                                        .map(|video_meta| Command::AddToQueue(video_meta.clone()))
                                }).collect::<Vec<_>>();
                                Ok(commands)
                            } else {
                                Err(String::from("Gone"))
                            }
                        } else {
                            Err(String::from("'list' field is needed"))
                        }
                    },
                    WebCommandType::PauseBeforeNext => {
                        Ok(vec![Command::PauseBeforeNext])
                    }
                    WebCommandType::QuitOnFinish => {
                        Ok(vec![Command::ToggleQuitOnFinish])
                    },
                    WebCommandType::Quit => {
                        Ok(vec![Command::Quit])
                    },
                    WebCommandType::ToggleSubtitles =>
                        Ok(vec![Command::ToggleDisplaySubtitles]),
                    WebCommandType::Announcement => {
                        if let Some(ref s) = web_command.text {
                            use chrono::Local;
                            if s.len() != 0 {
                                Ok(vec![Command::Announcement(s.clone(),Local::now())])
                            } else {
                                Err(String::from("field 'text' is present but empty"))
                            }
                        } else {
                            Err(String::from("'text' field is required"))
                        }
                    }
                };
                if let Ok(commands) = commands {
                    for command in commands {
                        if let Err(e) = tx.send(command) {
                            error!("An error happened when trying\
                            to send a command to the other thread : {}",e);
                            return Ok(Response::with(status::InternalServerError));
                        }
                    }
                    Ok(Response::with(status::NoContent))
                } else {
                    Ok(Response::with(status::BadRequest))
                }
            }
            Ok(&None) => Ok(Response::with(status::BadRequest)),
            Err(err) => Err(IronError::new(err, status::BadRequest)),
        }
    }

    fn logs() -> IronResult<Response> {
        use ::toyunda_player::log_messages::LOG_MESSAGES;
        let json_mime: Mime = "application/json".parse().unwrap();
        if let Ok(ref messages) = LOG_MESSAGES.read() {
            let messages = messages.iter()
                                   .map(|m| {
                format!("[{}] {}: {}",m.time.format("%F %T"),m.level,m.msg)
            })
                                   .collect::<Vec<_>>();
            match serde_json::to_string(&messages) {
                Ok(string) => {
                    Ok(Response::with((status::Ok, string, json_mime)))
                },
                Err(e) => {
                    error!("Error when displaying logs : {}",e);
                    // TODO replace this with an Error and handle Iron errors better
                    Ok(Response::with(status::InternalServerError))
                }
            }
        } else {
            Ok(Response::with((status::Gone, "{}", json_mime)))
        }
    }

    pub fn new<A: ToSocketAddrs>(address: A,
                                 toyunda_state: Weak<RwLock<ToyundaState>>,
                                 yaml_directories: Vec<PathBuf>)
                                 -> IronResult<Manager> {
        let mut yaml_files: Vec<VideoMeta> = Vec::new();
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
        }, "get_state");
        let tx_command = Mutex::new(tx);
        let weak_list = Arc::downgrade(&yaml_files);
        let weak_list2 = weak_list.clone();
        api_handler.post("command", move |request: &mut Request| {
            let tx_command = tx_command.lock().unwrap().clone();
            Self::command(request, tx_command, weak_list2.clone())
        }, "do_command");
        api_handler.get("listing", move |_r: &mut Request| {
            Self::list_request(weak_list.clone())
        }, "get_listing");
        api_handler.get("logs", move |_r: &mut Request| {
            Self::logs()
        }, "get_logs");
        let mut mount = Mount::new();
        mount.mount("/", Static::new("web/"));
        mount.mount("/api", api_handler);
        let listening = Iron::new(mount).http(address).unwrap();
        Ok(Manager {
            listening: listening,
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
