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
    // init the logger
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

    // INIT SDL
    let sdl_context = sdl2::init().unwrap();
    let mut video_subsystem = sdl_context.video().unwrap();
    let renderer = init::init_sdl(&mut video_subsystem);
    let video_subsystem_ptr = &mut video_subsystem as *mut _ as *mut c_void;
    // INIT MPV
    let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
    mpv_builder.set_option("sid", "no").unwrap(); // disables subtitles if any
    mpv_builder.set_option("softvol", "yes").unwrap(); // enables softvol so it can go higher than 100%
    mpv_builder.set_option("softvol-max", 250.0).unwrap(); // makes the max volume at 250%
    mpv_builder.try_hardware_decoding().unwrap(); // try hardware decoding instead of software decoding
    let mut mpv = mpv_builder.build_with_gl(Some(init::get_proc_address), video_subsystem_ptr)
       .expect("Error while initializing MPV");
    // BIND MPV WITH SDL

    let displayer = display::Displayer::new(renderer)
                        .expect("Failed to init displayer");
    // Create a new displayer for the toyunda_player

    let mut toyunda_player = ToyundaPlayer::new(mpv, displayer);
    toyunda_player.start(matches);
    let res = toyunda_player.main_loop(&sdl_context);
    match res {
        Ok(_) => {
            info!("Toyunda Player finished gracefully");
        },
        Err(e) => {
            error!("An uncoverable error occured : {}",e);
        }
    };
}
