use ::toyunda_player::ToyundaOptions;
use mpv::{MpvHandlerWithGl,MpvHandler,Event as MpvEvent};
use std::path::Path;
use ::subtitles::Subtitles;
use ::display::Displayer;
use sdl2::event::Event;
use sdl2::Sdl;
use sdl2::keyboard::{KeyboardState,Scancode,Keycode};
use std::cmp::{min,max};

pub struct ToyundaPlayer<'a> {
    subtitles:Option<Subtitles>,
    mpv:Box<MpvHandlerWithGl>,
    displayer:Displayer<'a>,
    options:ToyundaOptions
}

fn speed_btn(mpv: &mut MpvHandler, is_shift_pressed: bool, keynumber: u64) {
    let mut speed = (keynumber as f64) / 10.0;
    if (is_shift_pressed) {
        speed += 1.0;
    }
    mpv.set_property_async("speed", speed, 1).expect("Failed to modify player speed");
}

fn add_volume(mpv: &mut MpvHandler, delta: i64) {
    let max_volume: i64 = mpv.get_property("volume-max").expect("Failed to get volume-max");
    let current_volume: i64 = mpv.get_property("volume").expect("Failed to get volume");
    let new_volume = min(max(current_volume + delta, 0), max_volume);
    mpv.set_property("volume", new_volume).unwrap();
}

/// returns 3 boolean : (AltPressed,CtrlPressed,ShiftPressed)
#[inline]
fn get_alt_keys(keyboard_state:KeyboardState) -> (bool,bool,bool) {
    (keyboard_state.is_scancode_pressed(Scancode::RAlt) ||
     keyboard_state.is_scancode_pressed(Scancode::LAlt),
     keyboard_state.is_scancode_pressed(Scancode::RCtrl) ||
     keyboard_state.is_scancode_pressed(Scancode::LCtrl),
     keyboard_state.is_scancode_pressed(Scancode::RShift) ||
     keyboard_state.is_scancode_pressed(Scancode::LShift))
}

pub enum ToyundaAction {
    Nothing,
    Terminate
}

impl<'a> ToyundaPlayer<'a> {
    pub fn new(mpv_box:Box<MpvHandlerWithGl>,displayer:Displayer<'a> ) -> ToyundaPlayer<'a> {
        ToyundaPlayer {
            subtitles:None,
            mpv:mpv_box,
            displayer:displayer,
            options:ToyundaOptions::default()
        }
    }

    pub fn get_subtitles<P:AsRef<Path>>(&mut self,folder:P) {
        unimplemented!()
    }

    pub fn main_loop(&mut self,sdl_context:&Sdl) {
        let mut event_pump = sdl_context.event_pump().expect("Failed to create event_pump");
        'main: loop {
            let alt_keys = get_alt_keys(event_pump.keyboard_state());
            for event in event_pump.poll_iter() {
                match self.handle_event(event,alt_keys) {
                    Ok(ToyundaAction::Nothing) => {},
                    Ok(ToyundaAction::Terminate) => {break 'main},
                    Err(e) => {
                        error!("An error '{}' occured",e);
                    }
                };
            }
            while let Some(event) = self.mpv.wait_event(0.0) {
                match event {
                    MpvEvent::Shutdown | MpvEvent::EndFile(_) => {
                        break 'main;
                    }
                    _ => {}
                };
            }
            let (width, height) = self.displayer.sdl_renderer().window().unwrap().size();
            self.mpv.draw(0, width as i32, -(height as i32)).expect("Failed to draw");
            // displayer.display("0123456789ABCDEF0123456789abcdef0123456789");
            let _time_pos: Option<f64> = self.mpv.get_property("time-pos").ok();
            let frame_pos: Option<u32> = self.mpv.get_property::<i64>("estimated-frame-number")
                                            .ok()
                                            .map(|v| v as u32);
            self.displayer.render();
        }
    }

    pub fn handle_event(&mut self,event:Event,alt_keys_state:(bool,bool,bool)) -> Result<ToyundaAction,String> {
        let (_is_alt_pressed,is_ctrl_pressed,is_shift_pressed) = alt_keys_state;
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                Ok(ToyundaAction::Terminate),
            Event::KeyDown { keycode: Some(Keycode::Space),repeat: false, .. } => {
                match self.mpv.get_property("pause").unwrap() {
                    true => {
                        self.mpv.set_property_async("pause", false, 1)
                           .expect("Failed to pause player");
                    }
                    false => {
                        self.mpv.set_property_async("pause", true, 1)
                           .expect("Failed to unpause player");
                    }
                };
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::A), repeat: false, .. } => {
                println!("estimated_frame:\t{}\
                        \tcurrent_time:\t{}",
                        self.mpv.get_property::<i64>("estimated-frame-number").unwrap_or(0),
                        self.mpv.get_property::<f64>("time-pos").unwrap_or(0.0));
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 9);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 8);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 7);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 6);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 5);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 4);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 3);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 2);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 1);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } => {
                speed_btn(&mut self.mpv, is_shift_pressed, 10);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::KpPlus),  .. } => {
                add_volume(&mut self.mpv, 5);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::KpMinus), .. } => {
                add_volume(&mut self.mpv, -5);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } => {
                if (is_ctrl_pressed) {
                    match self.mpv.command(&["frame-step"]) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to frame step with error {:?}", err);
                        }
                    };
                } else if (is_shift_pressed) {
                    self.mpv.command(&["seek", 15.to_string().as_str()]).unwrap();
                } else {
                    self.mpv.command(&["seek", 3.to_string().as_str()]).unwrap();
                };
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } => {
                if (is_ctrl_pressed) {
                    match self.mpv.command(&["frame-back-step"]) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to frame back step with error {:?}", err);
                        }
                    };
                } else if (is_shift_pressed) {
                    self.mpv.command(&["seek", (-15).to_string().as_str()]).unwrap();
                } else {
                    self.mpv.command(&["seek", (-3).to_string().as_str()]).unwrap();
                };
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                if is_ctrl_pressed {
                    add_volume(&mut self.mpv, 5);
                };
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                if is_ctrl_pressed {
                    add_volume(&mut self.mpv, -5);
                };
                Ok(ToyundaAction::Nothing)
            }
            Event::MouseWheel { y:delta_y, .. } => {
                let delta_y: i64 = (delta_y as i64) *
                                   if is_ctrl_pressed {
                    1
                } else {
                    10
                };
                add_volume(&mut self.mpv, delta_y);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::F), repeat: false, .. } => {
                use sdl2::video::FullscreenType;
                let new_fullscreen_type =
                    match self.displayer.sdl_renderer().window().unwrap().fullscreen_state() {
                        FullscreenType::True | FullscreenType::Desktop => { // TODO warn if 'True'
                            FullscreenType::Off
                        },
                        FullscreenType::Off => {
                            FullscreenType::Desktop
                        }
                    };
                self.displayer.sdl_renderer_mut()
                              .window_mut()
                              .unwrap()
                              .set_fullscreen(new_fullscreen_type)
                              .expect("Failed to change fullscreen parameter of toyunda-player");
                Ok(ToyundaAction::Nothing)
            }
            _ => Ok(ToyundaAction::Nothing)
        }
    }
}
