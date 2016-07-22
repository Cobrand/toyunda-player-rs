use ::toyunda_player::ToyundaOptions;
use mpv::{MpvHandlerWithGl,MpvHandler,Event as MpvEvent};
use std::path::Path;
use ::subtitles::Subtitles;
use ::display::Displayer;
use sdl2::event::Event;
use sdl2::Sdl;
use sdl2::keyboard::{KeyboardState,Scancode,Keycode};
use std::cmp::{min,max};
use ::toyunda_player::error::*;
use ::toyunda_player::command::*;

pub struct ToyundaPlayer<'a> {
    subtitles:Option<Subtitles>,
    mpv:Box<MpvHandlerWithGl>,
    displayer:Displayer<'a>,
    options:ToyundaOptions
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

    pub fn import_subtitles<P:AsRef<Path>>(&mut self,path:P) -> Result<()> {
        let path : &Path = path.as_ref();
        match (path.is_file(),path.is_dir()) {
            (true,false) => {
                // is a file
                let lyr_path = path.with_extension("lyr");
                let frm_path = path.with_extension("frm");
                if (lyr_path.is_file() && frm_path.is_file()) {
                    info!("Loading subtitles ...");
                    Subtitles::load_from_lyr_frm(lyr_path,frm_path)
                        .map(|subtitles| {
                            self.subtitles = Some(subtitles);
                            ()
                        })
                        .map_err(|s| Error::Text(s))
                } else if lyr_path.is_file() {
                    error!("Could not find .frm file");
                    Err(Error::FileNotFound(frm_path.display().to_string()))
                } else if frm_path.is_file() {
                    error!("Could not find .lyr file");
                    Err(Error::FileNotFound(frm_path.display().to_string()))
                } else {
                    error!("Could not find .lyr and .frm file");
                    Err(Error::FileNotFound(lyr_path.display().to_string()))
                }
            },
            (false,true) => {
                // is a directory
                error!("Directories are not yet implemented");
                unimplemented!()
            },
            (false,false) => {
                // not a file, nor a directory
                error!("{} is not a file nor a directory, aborting parsing",path.display());
                Err(Error::Text(format!("{} is not a file nor a directory, aborting",path.display())))
            },
            (true,true) => {
                error!("File is both a file and a directory ... this should not happen");
                unreachable!()
            }
        }
    }

    pub fn render_frame(&mut self) -> Result<()> {
        use sdl2::rect::Rect;
        use std::io::{self, Write};
        let (width, height) = self.displayer.sdl_renderer().window().unwrap().size();
        self.mpv.draw(0, width as i32, -(height as i32)).expect("failed to draw frame with mpv");
        if let Some(ref subtitles) = self.subtitles {
            let frame_number : i64 = self.mpv.get_property("estimated-frame-number").unwrap_or(0);
            let subtitles_texture = subtitles.get_texture_at_frame(&mut self.displayer, frame_number as u32).unwrap();
            let (w,h) = self.displayer.sdl_renderer().output_size().expect("Failed to get render size");
            self.displayer.sdl_renderer_mut().set_blend_mode(::sdl2::render::BlendMode::Blend);
            self.displayer.sdl_renderer_mut().copy(&subtitles_texture,Some(Rect::new(0,0,w,h)),Some(Rect::new(0,0,w,h)));
        }
        self.displayer.render();
        Ok(())
    }

    pub fn main_loop(&mut self,sdl_context:&Sdl) -> Result<()> {
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
                let res = match event {
                    MpvEvent::Shutdown => Ok(ToyundaAction::Terminate),
                    MpvEvent::EndFile(_) => { // TODO remove EndFile and handle this better
                        self.execute_command(Command::EndFile)
                    },
                    _ => Ok(ToyundaAction::Nothing)
                };
                match res {
                    Ok(ToyundaAction::Nothing) => {},
                    Ok(ToyundaAction::Terminate) => {break 'main},
                    Err(e) => {
                        error!("An error '{}' occured",e);
                    }
                }
            }
            try!(self.render_frame());
        };
        Ok(())
    }

    pub fn handle_event(&mut self,event:Event,alt_keys_state:(bool,bool,bool)) -> Result<ToyundaAction> {
        let (_is_alt_pressed,is_ctrl_pressed,is_shift_pressed) = alt_keys_state;
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                Ok(ToyundaAction::Terminate),
            Event::KeyDown { keycode: Some(Keycode::Space),repeat: false, .. }
                => self.execute_command(Command::TogglePause),
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.9)),
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.9)),
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.8)),
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.8)),
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.7)),
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.7)),
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.6)),
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.6)),
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.5)),
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.5)),
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.4)),
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.4)),
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.3)),
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.3)),
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.2)),
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.2)),
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.1)),
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(0.1)),
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } if is_shift_pressed
                => self.execute_command(Command::SetSpeed(2.0)),
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. }
                => self.execute_command(Command::SetSpeed(1.0)),
            Event::KeyDown { keycode: Some(Keycode::KpPlus),  .. }
                => self.execute_command(Command::AddVolume(5)),
            Event::KeyDown { keycode: Some(Keycode::KpMinus), .. }
                => self.execute_command(Command::AddVolume(-5)),
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } if is_ctrl_pressed
                => self.execute_command(Command::Framestep(1)),
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } if is_shift_pressed
                => self.execute_command(Command::Seek(15.0)),
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. }
                => self.execute_command(Command::Seek(3.0)),
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } if is_ctrl_pressed
                => self.execute_command(Command::Framestep(-1)),
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } if is_shift_pressed
                => self.execute_command(Command::Seek(-15.0)),
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. }
                => self.execute_command(Command::Seek(-3.0)),
            Event::MouseWheel { y:delta_y, .. } if is_ctrl_pressed
                => self.execute_command(Command::AddVolume(delta_y as i64)),
            Event::MouseWheel { y:delta_y, .. }
                => self.execute_command(Command::AddVolume(10 * delta_y as i64)),
            Event::KeyDown { keycode: Some(Keycode::F), repeat: false, .. } =>
                self.execute_command(Command::ToggleFullscreen),
            _ => Ok(ToyundaAction::Nothing)
        }
    }

    pub fn subtitles(&self) -> Option<&Subtitles> {
        self.subtitles.as_ref()
    }

    pub fn subtitles_mut(&mut self) -> Option<&mut Subtitles> {
        self.subtitles.as_mut()
    }

    pub fn mpv(&self) -> &MpvHandlerWithGl {
        self.mpv.as_ref()
    }

    pub fn mpv_mut(&mut self) -> &mut MpvHandlerWithGl {
        self.mpv.as_mut()
    }

    pub fn displayer(&self) -> &Displayer<'a> {
        &self.displayer
    }

    pub fn displayer_mut(&mut self) -> &mut Displayer<'a> {
        &mut self.displayer
    }

    pub fn toyunda_options(&self) -> &ToyundaOptions {
        &self.options
    }

    pub fn toyunda_options_mut(&mut self) -> &mut ToyundaOptions {
        &mut self.options
    }
}