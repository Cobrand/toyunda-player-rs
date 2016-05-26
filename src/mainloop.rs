extern crate mpv ;
extern crate sdl2;

use sdl2::Sdl;
use sdl2::event::Event as SdlEvent;
use sdl2_sys::video::SDL_WindowFlags;
use sdl2::video::FullscreenType;
use sdl2::keyboard::{Keycode,Scancode};
use displayer::Displayer;
use std::cmp::{min,max};

fn speed_btn(mpv:&mut mpv::MpvHandler,is_shift_pressed:bool,keynumber:u64){
    let mut speed = (keynumber as f64) / 10.0 ;
    if (is_shift_pressed) {
        speed += 1.0;
    }
    mpv.set_property_async("speed",speed,1).expect("Failed to modify player speed");
}

fn add_volume(mpv:&mut mpv::MpvHandler,delta:i64){
    let max_volume : i64 = mpv.get_property("volume-max").expect("Failed to get volume-max");
    let current_volume : i64 = mpv.get_property("volume").expect("Failed to get volume");
    let new_volume = min(max(current_volume + delta,0),max_volume);
    mpv.set_property("volume",new_volume).unwrap();
}

fn example(time_pos:Option<f64>,displayer:&mut Displayer){
    use sdl2::pixels::Color;
    use display::{self,Text2D,Display};
    match time_pos {
        Some(time_pos) => {
            let mut alpha : u8 = 0 ;
            if (time_pos > 1.5 && time_pos <= 2.0 ){
                alpha = ((2.0 - time_pos) / (1.5 - 2.0) * 255.0 ) as u8 ;
            } else if (time_pos > 2.0 && time_pos < 5.0 ){
                alpha = 0xFF;
            } else if (time_pos >= 5.0 && time_pos < 5.5 ) {
                alpha = ((5.0 - time_pos) / (5.5 - 5.0) * 255.0 ) as u8 ;
            }
            let (r,g,b) : (u8,u8,u8) = (64,255,0);
            display::Text2D {
                text:vec![
                    display::TextElement {
                        text:"SALUUUUT".to_string(),
                        color:Color::RGBA(r,g,b,alpha),
                        outline:Some(Color::RGB(0,0,0)),
                        shadow:None
                    }
                ],
                size:display::Size::FitPercent(Some(0.95),Some(0.1)),
                pos:(display::PosX::Centered,display::PosY::FromTopPercent(0.01)),
                anchor:(0.5,0.0)
            }.draw(displayer);
        },
        None => {}
    }
}

pub fn main_loop(sdl_context:Sdl,mut displayer:Displayer,mut mpv:mpv::MpvHandler){
    let mut event_pump = sdl_context.event_pump().expect("Failed to create event_pump");
    'main: loop {
        let (is_alt_pressed,is_ctrl_pressed,is_shift_pressed) = {
            let keyboard_state = event_pump.keyboard_state();
            (
                keyboard_state.is_scancode_pressed(Scancode::RAlt) || keyboard_state.is_scancode_pressed(Scancode::LAlt),
                keyboard_state.is_scancode_pressed(Scancode::RCtrl) || keyboard_state.is_scancode_pressed(Scancode::LCtrl),
                keyboard_state.is_scancode_pressed(Scancode::RShift) || keyboard_state.is_scancode_pressed(Scancode::LShift)
            )
        };
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
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 9)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 8)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 7)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 6)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 5)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 4)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 3)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 2)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 1)},
                SdlEvent::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } => {speed_btn(&mut mpv, is_shift_pressed, 10)},
                SdlEvent::KeyDown { keycode: Some(Keycode::KpPlus),  .. } => {add_volume(&mut mpv, 5 );},
                SdlEvent::KeyDown { keycode: Some(Keycode::KpMinus), .. } => {add_volume(&mut mpv, -5);},
                SdlEvent::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } => {
                    if (is_ctrl_pressed){
                        match mpv.command(&["frame-step"]) {
                            Ok(_) => {},
                            Err(err) => {error!("Failed to frame step with error {:?}",err);}
                        };
                    } else if (is_shift_pressed) {
                        mpv.command(&["seek",15.to_string().as_str()]).unwrap();
                    } else {
                        mpv.command(&["seek",3.to_string().as_str()]).unwrap();
                    }
                }
                SdlEvent::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } => {
                    if (is_ctrl_pressed){
                        match mpv.command(&["frame-back-step"]) {
                            Ok(_) => {},
                            Err(err) => {error!("Failed to frame back step with error {:?}",err);}
                        };
                    } else if (is_shift_pressed) {
                        mpv.command(&["seek",(-15).to_string().as_str()]).unwrap();
                    } else {
                        mpv.command(&["seek",(-3).to_string().as_str()]).unwrap();
                    }
                }
                SdlEvent::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    if is_ctrl_pressed {
                        add_volume(&mut mpv, 5);
                    }
                }
                SdlEvent::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    if is_ctrl_pressed {
                        add_volume(&mut mpv, -5);
                    }
                }
                SdlEvent::MouseWheel { y:delta_y, .. } => {
                    let delta_y : i64 = (delta_y as i64) * if is_ctrl_pressed {1} else {10};
                    add_volume(&mut mpv, delta_y);
                }
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
        //displayer.display("0123456789ABCDEF0123456789abcdef0123456789");
        let time_pos : Option<f64> = mpv.get_property("time-pos").ok();
        let frame_pos : Option<u32> = mpv.get_property::<i64>("estimated-frame-number").ok().map(|v| v as u32 );
        //example(time_pos,&mut displayer);
        //displayer.example();
        match frame_pos {
            Some(frame_pos) => displayer.render_subtitles(frame_pos),
            _ => {}
        };
        displayer.render();
    }
}
