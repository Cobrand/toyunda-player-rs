extern crate mpv ;
extern crate sdl2;

use sdl2::Sdl;
use sdl2::event::Event as SdlEvent;
use sdl2_sys::video::SDL_WindowFlags;
use sdl2::video::FullscreenType;
use sdl2::keyboard::Keycode;
use displayer::Displayer;

pub fn main_loop(sdl_context:Sdl,mut displayer:Displayer,mut mpv:mpv::MpvHandler){
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
                    if (displayer.sdl_renderer().window().unwrap().window_flags() &
                        (SDL_WindowFlags::SDL_WINDOW_FULLSCREEN as u32)) != 0 {
                        displayer.sdl_renderer_mut().window_mut().unwrap().set_fullscreen(FullscreenType::Off)
                    } else {
                        displayer.sdl_renderer_mut().window_mut().unwrap().set_fullscreen(FullscreenType::Desktop)
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
        let (width, height) = displayer.sdl_renderer().window().unwrap().size();
        mpv.draw(0, width as i32, -(height as i32)).expect("Failed to draw");
        displayer.display("hajimete kara mada wasurenai desho' YOUR DREAM");
        displayer.render();
    }
}
