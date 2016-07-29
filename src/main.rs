#![allow(unused_parens)]
extern crate mpv ;
extern crate sdl2;
extern crate sdl2_ttf;
extern crate sdl2_image;
extern crate sdl2_sys;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
use clap::{Arg, App};

mod utils;
mod display;
mod init;
mod subtitles;
mod toyunda_player;

use toyunda_player::ToyundaPlayer;
// use mpv::mpv;
use std::env;
use std::path::Path;
use std::os::raw::c_void;

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("Toyunda Player RS")
                          .version("0.2")
                          .author("Cobrand")
                          .about("Flexible karaoke player")
                          .arg(Arg::with_name("VIDEO_FILE")
                            .help("Sets the video file(s) to play")
                            .multiple(true)
                            .required(true))
                          .get_matches();
    let mut video_path = String::new();
    if let Some(video_files) = matches.values_of("VIDEO_FILE") {
        for value in video_files {
            video_path = String::from(value);
            break;
        }
    } else {
        unreachable!()
    }
    // INIT SDL
    let sdl_context = sdl2::init().unwrap();
    let mut video_subsystem = sdl_context.video().unwrap();
    let renderer = init::init_sdl(&mut video_subsystem);
    let video_subsystem_ptr = &mut video_subsystem as *mut _ as *mut c_void;
    // INIT MPV
    let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
    mpv_builder.set_option("osc", true).unwrap();
    mpv_builder.set_option("sid", "no").unwrap();
    mpv_builder.set_option("softvol", "yes").unwrap();
    mpv_builder.set_option("softvol-max", 250.0).unwrap();
    mpv_builder.try_hardware_decoding().unwrap();
    let mut mpv = mpv_builder.build_with_gl(Some(init::get_proc_address), video_subsystem_ptr)
       .expect("Error while initializing MPV");
    // BIND MPV WITH SDL

    let displayer = display::Displayer::new(renderer)
                        .expect("Failed to init displayer");

    mpv.command(&["loadfile", &*video_path])
       .expect("Error loading file");

    let mut toyunda_player = ToyundaPlayer::new(mpv, displayer);
    match toyunda_player.import_subtitles(video_path) {
        Ok(_) => {
        },
        Err(e) => {
            error!("Error was received when importing subtitles : {}",e);
            warn!("Failed to import subtitles; File will play without subtitles")
        }
    };
    let res = toyunda_player.main_loop(&sdl_context);
    match res {
        Ok(_) => {},
        Err(e) => {
            error!("An uncoverable error occured : {}",e);
        }
    };
}
