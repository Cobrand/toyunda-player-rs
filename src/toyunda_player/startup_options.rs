use clap::ArgMatches;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use super::ToyundaMode;

#[derive(Debug)]
pub struct StartupOptions {
    pub manager_listen_address: Option<IpAddr>,
    pub manager_listen_port: Option<u16>,
    pub volume: Option<f64>,
    pub quit: Option<bool>,
    pub video_files: Vec<PathBuf>,
    pub songs_history: Option<PathBuf>,
    pub lookup_directories: Vec<PathBuf>,
    pub fullscreen: bool,
    pub mode: Option<ToyundaMode>,
    pub no_manager: bool,
}

#[derive(Debug)]
pub struct StartupParameters {
    pub manager_listen_address: IpAddr,
    pub manager_listen_port: u16,
    pub volume: Option<f64>,
    pub quit: Option<bool>,
    pub video_files: Vec<PathBuf>,
    pub songs_history: Option<PathBuf>,
    pub lookup_directories: Vec<PathBuf>,
    pub fullscreen: bool,
    pub mode: ToyundaMode,
    pub no_manager: bool,
}

impl StartupOptions {
    pub fn from_args<'a>(arg_matches: ArgMatches<'a>) -> Result<StartupOptions, String> {
        Ok(StartupOptions {
            manager_listen_address: match arg_matches.value_of("manager_listen_address") {
                Some(listen_address_str) => {
                    Some(try!(listen_address_str.parse::<IpAddr>().map_err(|e| format!("{}", e))))
                }
                None => None,
            },
            manager_listen_port: match arg_matches.value_of("manager_port") {
                Some(port_str) => Some(try!(port_str.parse::<u16>().map_err(|e| format!("{}", e)))),
                None => None,
            },
            volume: match arg_matches.value_of("volume") {
                Some(volume_str) => {
                    Some(try!(volume_str.parse::<f64>().map_err(|e| format!("{}", e))))
                }
                None => None,
            },
            quit: match (arg_matches.is_present("quit"), arg_matches.is_present("no-quit")) {
                (true, _) => Some(true),
                (_, true) => Some(false),
                _ => None,
            },
            video_files: arg_matches.values_of("VIDEO_FILE")
                .map(|d| {
                    d.map(|v| PathBuf::from(v))
                        .collect::<Vec<PathBuf>>()
                })
                .unwrap_or(vec![]),
            songs_history: arg_matches.value_of("songs_history").map(|s| PathBuf::from(s)),
            lookup_directories: arg_matches.values_of("yaml_directories")
                .map(|d| {
                    d.map(|v| PathBuf::from(v))
                        .collect::<Vec<PathBuf>>()
                })
                .unwrap_or(vec![]),
            fullscreen: arg_matches.is_present("fullscreen"),
            mode: match (arg_matches.is_present("edit_mode"),
                         arg_matches.is_present("karaoke_mode")) {
                (true, false) => Some(ToyundaMode::EditMode),
                (false, true) => Some(ToyundaMode::KaraokeMode),
                _ => None,
            },
            no_manager: arg_matches.is_present("no_manager"),
        })
    }

    pub fn to_params(self) -> StartupParameters {
        StartupParameters {
            manager_listen_address: self.manager_listen_address
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            manager_listen_port: self.manager_listen_port.unwrap_or(8080),
            volume: self.volume,
            quit: self.quit,
            video_files: self.video_files,
            songs_history: self.songs_history,
            lookup_directories: self.lookup_directories,
            fullscreen: self.fullscreen,
            mode: self.mode.unwrap_or(ToyundaMode::NormalMode),
            no_manager: self.no_manager,
        }
    }
}
