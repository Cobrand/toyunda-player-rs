extern crate serde_json;

use ::toyunda_player::*;
use mpv::{MpvHandlerWithGl,Event as MpvEvent};
use std::path::{Path,PathBuf};
use ::subtitles::Subtitles;
use ::display::Displayer;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::Sdl;
use sdl2::keyboard::{KeyboardState,Scancode,Keycode};
use std::sync::{RwLock,Arc};
use ::toyunda_player::error::*;
use ::toyunda_player::command::*;
use ::toyunda_player::state::*;
use ::toyunda_player::playing_state::*;
use ::toyunda_player::manager::*;
use mpv::EndFileReason::MPV_END_FILE_REASON_EOF ;
use clap::ArgMatches ;

pub struct ToyundaPlayer<'a> {
    subtitles:Option<Subtitles>,
    mpv:Box<MpvHandlerWithGl>,
    displayer:Displayer<'a>,
    options:ToyundaOptions,
    state:Arc<RwLock<State>>,
    graphic_messages:Vec<graphic_message::GraphicMessage>
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

#[derive(Debug,Clone)]
pub enum ToyundaAction {
    Nothing,
    PlayNext,
    Terminate
}

impl<'a> ToyundaPlayer<'a> {
    pub fn new(mpv_box:Box<MpvHandlerWithGl>,displayer:Displayer<'a> ) -> ToyundaPlayer<'a> {
        ToyundaPlayer {
            subtitles:None,
            mpv:mpv_box,
            displayer:displayer,
            options:ToyundaOptions::default(),
            state: Arc::new(RwLock::new(State {
                playlist: Playlist::new(),
                playing_state: PlayingState::Idle,
                logs: Vec::new()
            })),
            graphic_messages:Vec::with_capacity(2)
        }
    }

    pub fn start(&mut self,arg_matches:ArgMatches) -> Result<()> {
        if let Some(video_files) = arg_matches.values_of("VIDEO_FILE") {
            let mut state = self.state().write().unwrap();
            for value in video_files {
                match VideoMeta::new(value) {
                    Ok(video_meta) => {
                        state.playlist.push_back(video_meta);
                    },
                    Err(e) => {
                        error!("Error when importing file : '{}'",e);
                    }
                }
            }
        }

        if arg_matches.is_present("quit") {
            self.options.quit_when_finished = Some(true);
        } else if arg_matches.is_present("no-quit") {
            self.options.quit_when_finished = Some(false);
        }

        if arg_matches.is_present("karaoke_mode") {
            self.options.mode = ToyundaMode::KaraokeMode;
            info!("Enabling karaoke mode");
        } else if arg_matches.is_present("edit_mode") {
            self.options.mode = ToyundaMode::EditMode;
            info!("Enabling edit mode");
        } else {
            self.options.mode = ToyundaMode::NormalMode;
        }

        if let Some(volume) = arg_matches.value_of("volume") {
            match volume.parse::<f64>() {
                Ok(volume) => {
                    match self.mpv.set_property("volume",volume) {
                        Ok(_) => {
                            info!("Successfully override initial volume");
                        },
                        Err(e) => {
                            error!("Could not change initial volume of mpv, error '{}' ({:?})",e,e);
                        }
                    };
                },
                Err(e) => {
                    error!("Error when parsing volume, expected some float, got '{}'; (error {:?})",volume,e);
                }
            }
        }
        Ok(())
    }

    pub fn add_graphic_message(&mut self,category:graphic_message::Category,message:&str) {
        use std::time::{Instant,Duration};
        self.graphic_messages.push( graphic_message::GraphicMessage {
            category:category,
            text: String::from(message),
            up_until : Instant::now() + Duration::from_secs(5)
        });
    }

    pub fn reload_subtitles(&mut self) -> Result<()> {
        let path = match &self.state().read().unwrap().playing_state {
            &PlayingState::Idle => {
                return Err(Error::Text(String::from("Error when reloading subtitles : no file is playing")));
            },
            &PlayingState::Playing(ref video_meta) => {
                video_meta.video_path.clone()
            }
        };
        try!(self.import_subtitles(path));
        Ok(())
    }

    pub fn import_subtitles<P:AsRef<Path>>(&mut self,path:P) -> Result<()> {
        let path : &Path = path.as_ref();
        match (path.is_file(),path.is_dir()) {
            (true,false) => {
                // is a file
                let json_path = path.with_extension("json");
                let lyr_path = path.with_extension("lyr");
                let frm_path = path.with_extension("frm");
                if (json_path.is_file()) {
                    info!("Loading {}",json_path.display());
                    let json_file = ::std::fs::File::open(json_path).expect("Failed to open JSON file");
                    let mut subtitles : Subtitles = serde_json::from_reader(json_file).expect("Failed to load json file");
                    subtitles.adjust_sentences_row();
                    self.subtitles = Some(subtitles);
                    Ok(())
                } else {
                    info!("Failed to load json file, trying lyr and frm files");
                    if (lyr_path.is_file() && frm_path.is_file()) {
                        info!("Loading subtitles with lyr and frm ...");
                        self.add_graphic_message(graphic_message::Category::Info, "Failed to load json subtitle file, loading lyr and frm");
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

    pub fn render_messages(&mut self) -> Result<()> {
        use ::toyunda_player::graphic_message::*;
        use std::time::Instant;
        use ::display::* ;
        let is_karaoke_mode =  self.options.mode == ToyundaMode::KaraokeMode ;
        self.graphic_messages.retain(|ref message| {
            message.up_until > Instant::now() &&
            (!is_karaoke_mode || message.category == graphic_message::Category::Announcement)
        });
        let message_height = 0.06 ;
        for (n,ref message) in self.graphic_messages.iter().enumerate() {
            let text_elt : TextElement = TextElement {
                text : message.text.clone(),
                attach_logo : false,
                color : match message.category {
                    Category::Error => Color::RGB(255,0,0),
                    Category::Warn => Color::RGB(255,140,0),
                    Category::Info => Color::RGB(0,255,255),
                    Category::Announcement => Color::RGB(255,255,255)
                },
                outline : Some(Color::RGB(0,0,0)),
                shadow : None
            };
            let text2d : Text2D = Text2D {
                text: vec![text_elt],
                size: Size::FitPercent(Some(0.98),Some(message_height)),
                pos: (PosX::FromLeftPercent(0.01),
                      PosY::FromBottomPercent(0.01 + message_height * n as f32)),
                anchor: (0.0, 1.0),
            };
            text2d.draw(&mut self.displayer);
        };
        Ok(())
    }

    pub fn render_frame(&mut self) -> Result<()> {
        use sdl2::rect::Rect;
        let (width, height) = self.displayer.sdl_renderer().window().unwrap().size();
        self.mpv.draw(0, width as i32, -(height as i32)).expect("failed to draw frame with mpv");
        if let Some(ref subtitles) = self.subtitles {
            if self.options.display_subtitles {
                let frame_number : i64 = self.mpv.get_property("estimated-frame-number").unwrap_or(0);
                let subtitles_texture = subtitles.get_texture_at_frame(&mut self.displayer, frame_number as u32).unwrap();
                let (w,h) = self.displayer.sdl_renderer().output_size().expect("Failed to get render size");
                self.displayer.sdl_renderer_mut().set_blend_mode(::sdl2::render::BlendMode::Blend);
                self.displayer.sdl_renderer_mut().copy(&subtitles_texture,Some(Rect::new(0,0,w,h)),Some(Rect::new(0,0,w,h)));
            }
        }
        try!(self.render_messages());
        self.displayer.render();
        Ok(())
    }

    pub fn main_loop(&mut self,sdl_context:&Sdl) -> Result<()> {
        use std::sync::mpsc::* ;

        let mut event_pump = sdl_context.event_pump().expect("Failed to create event_pump");
        let (tx,rx) = channel() ;
        let mut manager = Manager::new("0.0.0.0:8080",Arc::downgrade(&self.state),tx).unwrap();
        'main: loop {
            let alt_keys = get_alt_keys(event_pump.keyboard_state());
            for event in event_pump.poll_iter() {
                match self.handle_event(event,alt_keys) {
                    Ok(ToyundaAction::Nothing) => {},
                    Ok(ToyundaAction::Terminate) => {break 'main},
                    Ok(ToyundaAction::PlayNext) => {
                        match self.execute_command(Command::PlayNext) {
                            Err(e) => {
                                error!("An error '{}' occured when trying to play next file",e)
                            },
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("An error '{}' occured",e);
                    }
                };
            }
            for command in rx.try_recv() {
                let res = self.execute_command(command);
                match res {
                    Ok(ToyundaAction::Nothing) => {},
                    Ok(ToyundaAction::Terminate) => {break 'main},
                    Ok(ToyundaAction::PlayNext) => {
                        match self.execute_command(Command::PlayNext) {
                            Err(e) => {
                                error!("An error '{}' occured when trying to play next file",e)
                            },
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("An error '{}' occured ({:?})",e,e);
                    }
                }
            }
            while let Some(event) = self.mpv.wait_event(0.0) {
                let res = match event {
                    MpvEvent::Idle => {
                        self.execute_command(Command::PlayNext)
                    },
                    MpvEvent::Shutdown => Ok(ToyundaAction::Terminate),
                    MpvEvent::EndFile(Ok(MPV_END_FILE_REASON_EOF)) => { // TODO remove EndFile and handle this better
                        self.execute_command(Command::EndFile)
                    },
                    _ => Ok(ToyundaAction::Nothing)
                };
                match res {
                    Ok(ToyundaAction::Nothing) => {},
                    Ok(ToyundaAction::Terminate) => {break 'main},
                    Ok(ToyundaAction::PlayNext) => {
                        match self.execute_command(Command::PlayNext) {
                            Err(e) => {
                                error!("An error '{}' occured when trying to play next file",e)
                            },
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("An error '{}' occured ({:?})",e,e);
                    }
                }
            }
            try!(self.render_frame());
        };
        Ok(())
    }

    pub fn handle_event(&mut self,event:Event,alt_keys_state:(bool,bool,bool)) -> Result<ToyundaAction> {
        use ::toyunda_player::ToyundaMode::*;
        let (_is_alt_pressed,is_ctrl_pressed,is_shift_pressed) = alt_keys_state;
        let mode = self.options.mode ; // shortcut
        match event {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                Ok(ToyundaAction::Terminate),
            Event::KeyDown { keycode: Some(Keycode::Space),repeat: false, .. }
                => self.execute_command(Command::TogglePause),
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.9)),
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.9)),
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.8)),
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.8)),
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.7)),
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.7)),
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.6)),
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.6)),
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.5)),
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.5)),
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.4)),
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.4)),
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.3)),
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.3)),
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.2)),
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.2)),
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(1.1)),
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(0.1)),
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } if mode != KaraokeMode &&  is_shift_pressed
                => self.execute_command(Command::SetSpeed(2.0)),
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } if mode != KaraokeMode
                => self.execute_command(Command::SetSpeed(1.0)),
            Event::KeyDown { keycode: Some(Keycode::KpPlus),  .. }
                => self.execute_command(Command::AddVolume(5)),
            Event::KeyDown { keycode: Some(Keycode::KpMinus), .. }
                => self.execute_command(Command::AddVolume(-5)),
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } if mode != KaraokeMode && is_ctrl_pressed
                => self.execute_command(Command::Framestep(1)),
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } if mode != KaraokeMode && is_shift_pressed
                => self.execute_command(Command::Seek(15.0)),
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false,.. } if mode != KaraokeMode
                => self.execute_command(Command::Seek(3.0)),
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } if mode != KaraokeMode && is_ctrl_pressed
                => self.execute_command(Command::Framestep(-1)),
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } if mode != KaraokeMode && is_shift_pressed
                => self.execute_command(Command::Seek(-15.0)),
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false,.. } if mode != KaraokeMode
                => self.execute_command(Command::Seek(-3.0)),
            Event::KeyDown { keycode: Some(Keycode::R), repeat: false,.. } if mode != KaraokeMode
                => self.execute_command(Command::ReloadSubtitles),
            Event::KeyDown { keycode: Some(Keycode::N), repeat: false,.. }
                => self.execute_command(Command::PlayNext),
            Event::DropFile { filename ,.. }
                => self.execute_command(Command::AddToQueue(PathBuf::from(filename))),
            Event::KeyDown { keycode: Some(Keycode::S), repeat: false,.. } if mode != KaraokeMode
                => {
                match &self.state().read().unwrap().playing_state {
                    &PlayingState::Playing(ref video_meta) => {
                        if let Some(ref sub) = self.subtitles {
                            let json_file_path = video_meta.json_path();
                            let sub : Subtitles = sub.clone();
                            ::std::thread::spawn(move || {
                                let mut file = ::std::fs::File::create(&json_file_path)
                                    .expect("Failed to create subtitles file");
                                serde_json::to_writer_pretty(&mut file, &sub).unwrap();
                                info!("Saved file {}",json_file_path.display());
                            });
                        };
                    },
                    _ => {
                        error!("Could not save file, no video is playing !");
                    }
                }
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyDown { keycode: Some(Keycode::V), repeat: false,.. }
                => self.execute_command(Command::ToggleDisplaySubtitles),
            Event::MouseWheel { y:delta_y, .. } if is_ctrl_pressed
                => self.execute_command(Command::AddVolume(delta_y as i64)),
            Event::MouseWheel { y:delta_y, .. }
                => self.execute_command(Command::AddVolume(10 * delta_y as i64)),
            Event::KeyDown { keycode: Some(Keycode::F), repeat: false, .. } =>
                self.execute_command(Command::ToggleFullscreen),
            _ => Ok(ToyundaAction::Nothing)
        }
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

    pub fn state(&self) -> &RwLock<State> {
        &self.state
    }

    pub fn options(&self) -> &ToyundaOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut ToyundaOptions {
        &mut self.options
    }

    pub fn clear_subtitles(&mut self) {
        self.subtitles = None;
    }
}
