#![allow(unused_parens)]
#![feature(proc_macro)]
//#![feature(custom_derive, plugin)] // necessary for serde
#![feature(conservative_impl_trait)]
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;

extern crate iron ;
extern crate staticfile;
extern crate mount;
extern crate router;
extern crate bodyparser;

extern crate read_color;

#[macro_use]
extern crate lazy_static;

extern crate mpv;
extern crate sdl2;
extern crate sdl2_sys;
#[macro_use]
extern crate log;
extern crate fern;
extern crate chrono;
#[macro_use]
extern crate clap;
use clap::{Arg, App, SubCommand};

mod utils;
mod overlay;
mod sdl_displayer;
mod init;
mod subtitles;
mod toyunda_player;
mod mpv_plug;
mod update_json;

use update_json::update_json;

use toyunda_player::log_messages::{LOG_MESSAGES,LogMessage as ToyundaLogMessage};
use toyunda_player::StartupOptions;

fn main() {
    struct _DummyLog {};
    impl fern::Logger for _DummyLog {
        fn log(&self, msg:&str, level:&log::LogLevel, _:&log::LogLocation) -> Result<(),fern::LogError> {
            let level = level.clone();
            let msg = String::from(msg);
            let time = chrono::Local::now();
            std::thread::spawn(move || {
                if let Ok(mut v) = LOG_MESSAGES.write() {
                    v.push(ToyundaLogMessage {
                        level:level,
                        time:time,
                        msg:msg
                    });
                }
            });
            Ok(())
        }
    }
    let toyunda_log_path = ::std::env::current_exe().unwrap().with_file_name("toyunda.log");
    let fileout_config = fern::DispatchConfig {
        format: Box::new(|msg:&str, level: &log::LogLevel, _:&log::LogLocation|{
            format!("[{}][{}] {}",chrono::Local::now().format("%F %T"),level,msg)
        }),
        output: vec![fern::OutputConfig::file(&toyunda_log_path)],
        level: log::LogLevelFilter::Info
    };
    // init the logger
    let stdout_config = fern::DispatchConfig {
        format: Box::new(|msg:&str, level: &log::LogLevel, _:&log::LogLocation|{
            format!("[{}][{}] {}",chrono::Local::now().format("%F %T"),level,msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Warn
    };
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg:&str, _level: &log::LogLevel, _:&log::LogLocation|{
            String::from(msg)
        }),
        output: vec![
            fern::OutputConfig::child(stdout_config),
            fern::OutputConfig::child(fileout_config),
            fern::OutputConfig::custom(Box::new(_DummyLog{}))
        ],
        level: log::LogLevelFilter::Info
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        println!("Failed to initialize logger, no messages will be shown ! Error: {}",e);
    };
    let matches = App::new("Toyunda Player")
        .version(crate_version!())
        .after_help("PLAYER SHORTCUTS :\n    * V : Hides / Shows subtitles\n    * F : Toggles \
                     fullscreen\n    * + / - : Raises / Lowers the volume\n    * Left / Right \
                     arrow : Seek backwards / frontwards\n    ")
        .author("Cobrand")
        .about("A flexible karaoke player for the epitanime association")
        .arg(Arg::with_name("karaoke_mode")
            .short("k")
            .long("karaoke")
            .help("Sets mode to karaoke mode"))
        .arg(Arg::with_name("edit_mode")
            .short("e")
            .long("edit")
            .help("Sets mode to edit mode")
            .conflicts_with("karaoke_mode"))
        .arg(Arg::with_name("fullscreen")
            .short("f")
            .long("fullscreen")
            .help("Enables fullscreen"))
        .arg(Arg::with_name("yaml_directories")
            .short("d")
            .long("directory")
            .takes_value(true)
            .multiple(true)
            .help("Where to look the yaml files at")
            .conflicts_with("edit_mode"))
        .arg(Arg::with_name("songs_history")
            .short("s")
            .long("songs-history")
            .takes_value(true)
            .help("Where to look the songs history at"))
        .arg(Arg::with_name("manager_port")
            .short("p")
            .long("port")
            .help("The port the manager listens to, default is 8080")
            .takes_value(true))
        .arg(Arg::with_name("manager_listen_address")
            .long("listen-address")
            .takes_value(true)
            .help("Which address does this listen to, default is 0.0.0.0"))
        .arg(Arg::with_name("no_manager")
            .long("no-manager")
            .help("Prevents the manager from starting in karaoke or normal mode"))
        .arg(Arg::with_name("volume")
            .short("v")
            .long("volume")
            .takes_value(true)
            .help("Initial volume of the player; Default is 100"))
        .arg(Arg::with_name("quit")
            .short("q")
            .long("quit")
            .help("Forces quiting the player once the waiting queue is finished"))
        .arg(Arg::with_name("no-quit")
            .long("no-quit")
            .conflicts_with("quit")
            .help("Forces keeping alive the player once the waiting queue is finished"))
        .arg(Arg::with_name("VIDEO_FILE")
            .help("Sets the video file(s) to play")
            .use_delimiter(false)
            .multiple(true))
        .subcommand(SubCommand::with_name("update")
                    .about("updates a json file from .yaml data")
                    .arg(Arg::with_name("JSON_FILE")
                         .use_delimiter(false)
                         .required(true)))
        .get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("update") {
        if update_json(sub_matches) {
            ::std::process::exit(0);
        } else {
            ::std::process::exit(-1);
        }
    }
    let startup_options = match StartupOptions::from_args(matches) {
        Err(e) => {
            error!("Error when parsing command line parameters: {}",e);
            ::std::process::exit(1)
        },
        Ok(startup_options) => startup_options,
    };
    init::player_start(startup_options.to_params());
}
