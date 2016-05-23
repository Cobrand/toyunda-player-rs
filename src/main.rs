#![allow(unused_parens)]
extern crate mpv ;
extern crate sdl2;
extern crate sdl2_sys;
#[macro_use]
extern crate log;
extern crate env_logger;

mod displayer;
mod init;
mod mainloop;

// use mpv::mpv;
use std::env;
use std::path::Path;
use std::os::raw::c_void;

fn start_player(video_path: &Path) {
    // INIT SDL
    let sdl_context = sdl2::init().unwrap();
    let mut video_subsystem = sdl_context.video().unwrap();
    let renderer = init::init_sdl(&mut video_subsystem);
    let video_subsystem_ptr = &mut video_subsystem as *mut _ as *mut c_void;
    // INIT MPV
    let mut mpv = mpv::MpvHandler::create().expect("Error while creating MPV");
    mpv.set_option("osc",true).unwrap();
    mpv.set_option("sid","no").unwrap();
    mpv.set_option("softvol","yes").unwrap();
    mpv.set_option("softvol-max",200.0).unwrap();
    mpv.init_with_gl(Some(init::get_proc_address), video_subsystem_ptr).expect("Error while initializing MPV");
    // BIND MPV WITH SDL
    let displayer = displayer::Displayer::new(renderer).expect("Failed to init displayer");

    let video_path = video_path.to_str().expect("Expected a string for Path, got None");
    mpv.command(&["loadfile", video_path as &str])
       .expect("Error loading file");

    mainloop::main_loop(sdl_context,displayer,mpv);
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: ./sdl [any mp4, avi, mkv, ... file]");
    } else {
        let path: &Path = Path::new(&args[1]);
        if path.is_file() {
            start_player(path);
        } else {
            println!("A file is required; {} is not a valid file",
                     path.to_str().unwrap());
        }
    }
}
