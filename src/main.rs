extern crate mpv ;
extern crate sdl2;
extern crate sdl2_sys;
#[macro_use]
extern crate log;
extern crate env_logger;

mod init;
// use mpv::mpv;
use std::env;
use std::path::Path;
use std::os::raw::{c_void,c_char};
use std::ffi::CStr;
use sdl2_sys::video::SDL_WindowFlags;
use sdl2::video::FullscreenType;
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::Keycode;

fn sdl_example(video_path: &Path) {
    let opengl_driver = init::find_sdl_gl_driver().unwrap() as u32;
    let sdl_context = sdl2::init().unwrap();
    let mut video_subsystem = sdl_context.video().unwrap();
    let mut renderer = init::init_sdl(&mut video_subsystem,opengl_driver);
    let video_subsystem_ptr = &mut video_subsystem as *mut _ as *mut c_void;
    let mut mpv = mpv::MpvHandler::create().expect("Error while creating MPV");
    mpv.init_with_gl(Some(init::get_proc_address), video_subsystem_ptr).expect("Error while initializing MPV");

    let video_path = video_path.to_str().expect("Expected a string for Path, got None");
    mpv.command(&["loadfile", video_path as &str])
       .expect("Error loading file");

    let mut event_pump = sdl_context.event_pump().expect("Failed to create event_pump");
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit {..} | SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                SdlEvent::KeyDown { keycode: Some(Keycode::Space),repeat: false, .. } => {
                    match mpv.get_property("pause").unwrap() {
                        true => {mpv.set_property_async("pause",false,1).expect("Failed to pause player");},
                        false => {mpv.set_property_async("pause",true,1).expect("Failed to unpause player");}
                    }
                },
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } => {mpv.set_property_async("speed",0.9,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } => {mpv.set_property_async("speed",0.8,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } => {mpv.set_property_async("speed",0.7,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } => {mpv.set_property_async("speed",0.6,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } => {mpv.set_property_async("speed",0.5,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } => {mpv.set_property_async("speed",0.4,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } => {mpv.set_property_async("speed",0.3,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } => {mpv.set_property_async("speed",0.2,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } => {mpv.set_property_async("speed",0.1,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } => {mpv.set_property_async("speed",1.0,1).unwrap();},
                SdlEvent::KeyDown { keycode: Some(Keycode::F), repeat: false, .. } => {
                    if (renderer.window().unwrap().window_flags() &
                        (SDL_WindowFlags::SDL_WINDOW_FULLSCREEN as u32)) != 0 {
                        renderer.window_mut().unwrap().set_fullscreen(FullscreenType::Off)
                    } else {
                        renderer.window_mut().unwrap().set_fullscreen(FullscreenType::Desktop)
                    }
                    .expect("Failed to change fullscreen parameter of toyunda-player");
                }
                _ => {}
            }
        }
        while let Some(event) = mpv.wait_event(0.0) {
            match event {
                mpv::Event::Shutdown | mpv::Event::EndFile(_) => {
                    break 'main;
                }
                _ => {}
            };
        }
        let (width, height) = renderer.window().unwrap().size();
        if mpv.is_update_available(){
            mpv.draw(0, width as i32, -(height as i32)).expect("Failed to draw ");
        }
        renderer.window().unwrap().gl_swap_window();
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: ./sdl [any mp4, avi, mkv, ... file]");
    } else {
        let path: &Path = Path::new(&args[1]);
        if path.is_file() {
            sdl_example(path);
        } else {
            println!("A file is required; {} is not a valid file",
                     path.to_str().unwrap());
        }
    }
}
