#![allow(unused_parens)]
#![feature(custom_derive, plugin)] // necessary for serde
#![plugin(serde_macros)]
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

extern crate iron ;
extern crate staticfile;
extern crate mount;
extern crate router;
extern crate bodyparser;

#[macro_use]
extern crate lazy_static;

extern crate mpv;
extern crate sdl2;
extern crate sdl2_ttf;
extern crate sdl2_image;
extern crate sdl2_sys;
#[macro_use]
extern crate log;
extern crate fern;
extern crate chrono;
#[macro_use]
extern crate clap;
use clap::{Arg, ArgMatches, App, SubCommand};

mod utils;
mod overlay;
mod sdl_displayer;
mod init;
mod subtitles;
mod toyunda_player;
mod mpv_plug;
mod update_json;

use update_json::update_json;
use toyunda_player::ToyundaPlayer;
use sdl_displayer::SDLDisplayer;
use std::os::raw::c_void;
use toyunda_player::log_messages::{LOG_MESSAGES,LogMessage as ToyundaLogMessage};

fn player_start(matches:ArgMatches) {
    // INIT SDL
    let sdl_context = sdl2::init().unwrap();
    let mut video_subsystem = sdl_context.video().unwrap();
    let renderer = init::init_sdl(&mut video_subsystem, &matches);
    let video_subsystem_ptr = &mut video_subsystem as *mut _ as *mut c_void;
    // INIT MPV
    let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
    mpv_builder.set_option("sid", "no").unwrap(); // disables subtitles if any
    mpv_builder.set_option("softvol", "yes").unwrap(); // enables softvol so it can go higher than 100%
    mpv_builder.set_option("softvol-max", 250.0).unwrap(); // makes the max volume at 250%
    mpv_builder.set_option("aid",2i64).unwrap(); // aid to 2; normalized audio if there is one
    mpv_builder.try_hardware_decoding().unwrap(); // try hardware decoding instead of software decoding
    let mpv = mpv_builder.build_with_gl(Some(init::get_proc_address), video_subsystem_ptr)
        .expect("Error while initializing MPV");
    // BIND MPV WITH SDL

    let displayer = SDLDisplayer::new(renderer).expect("Failed to init displayer");

    if matches.is_present("karaoke_mode") {
        let mouse_utils = sdl_context.mouse();
        mouse_utils.show_cursor(false);
        // dont show cursor on top of player in karaoke mode
    }
    // Create a new displayer for the toyunda_player

    let mut toyunda_player = ToyundaPlayer::new(mpv, displayer);
    match toyunda_player.start(matches) {
        Err(e) => {
            error!("Failed to start player with given arguments, expect default parameters !\n\
                    '{}' ({:?})",
                   e,
                   e);
        }
        Ok(_) => {
            info!("Parsed arguments successfully");
        }
    };
    let res = toyunda_player.main_loop(&sdl_context);
    match res {
        Ok(_) => {
            info!("Toyunda Player finished gracefully");
        }
        Err(e) => {
            error!("An uncoverable error occured : {}", e);
        }
    };
}

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

    // init the logger
    let stdout_config = fern::DispatchConfig {
        format: Box::new(|msg:&str, level: &log::LogLevel, _:&log::LogLocation|{
            format!("[{}] {} : {}",chrono::Local::now().format("%F %T"),level,msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Warn
    };
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg:&str, level: &log::LogLevel, _:&log::LogLocation|{
            format!("[{}] {} : {}",chrono::Local::now().format("%F %T"),level,msg)
        }),
        output: vec![
            fern::OutputConfig::child(stdout_config),
            fern::OutputConfig::file("toyunda.log"),
            fern::OutputConfig::custom(Box::new(_DummyLog{}))
        ],
        level: log::LogLevelFilter::Info
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        println!("Failed to initialize logger, no messages will be shown ! Error : {}",e);
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
        .arg(Arg::with_name("yaml_directory")
            .short("d")
            .long("directory")
            .takes_value(true)
            .help("Where to look the yaml files at")
            .conflicts_with("edit_mode"))
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
    player_start(matches);
}
