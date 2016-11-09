extern crate serde_json;

use super::*;
use mpv::{MpvHandlerWithGl, Event as MpvEvent};
use std::path::PathBuf;
use ::subtitles::{Subtitles,Load,AsSentenceOptions};
use ::overlay::pos::*;
use ::overlay::{Display,OverlayFrame,TextUnit,TextSubUnit,Outline,Color,AlphaColor};
use ::sdl_displayer::{SDLDisplayer,SDLDisplayParameters as DisplayParams};
use sdl2::event::Event;
use sdl2::pixels::Color as SdlColor;
use sdl2::Sdl;
use sdl2::keyboard::{KeyboardState, Scancode, Keycode};
use std::sync::{RwLock, Arc};
use ::toyunda_player::error::*;
use ::toyunda_player::command::*;
use ::toyunda_player::state::*;
use ::toyunda_player::playing_state::*;
use ::toyunda_player::manager::*;
use ::toyunda_player::editor::*;
use ::toyunda_player::toyunda_history::*;
use ::utils::RGB;
use ::mpv_plug::MpvCache;
use mpv::EndFileReason::MPV_END_FILE_REASON_EOF;
use mpv::Error::MPV_ERROR_PROPERTY_UNAVAILABLE;
use clap::ArgMatches;
use chrono::{DateTime,Local};

pub struct ToyundaPlayer<'a> {
    pub subtitles: Option<Subtitles>,
    pub mpv: Box<MpvHandlerWithGl>,
    pub displayer: SDLDisplayer<'a>,
    pub options: ToyundaOptions,
    pub state: Arc<RwLock<State>>,
    pub manager: Option<Manager>,
    pub editor_state: Option<EditorState>,
    pub announcements: Vec<(String,DateTime<Local>)>,
    pub songs_history: Option<SongsHistory>,
    mpv_cache : MpvCache,
    unsaved_changes : bool,
}

/// returns 3 boolean : (AltPressed,CtrlPressed,ShiftPressed)
#[inline]
fn get_alt_keys(keyboard_state: KeyboardState) -> (bool, bool, bool) {
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
    Terminate,
}

impl<'a> ToyundaPlayer<'a> {
    pub fn new(mpv_box: Box<MpvHandlerWithGl>, displayer: SDLDisplayer<'a>) -> ToyundaPlayer<'a> {
        ToyundaPlayer {
            subtitles: None,
            mpv: mpv_box,
            displayer: displayer,
            options: ToyundaOptions::default(),
            state: Arc::new(RwLock::new(State {
                playlist: Playlist::new(),
                playing_state: PlayingState::Idle,
                display_subtitles: true,
                quit_when_finished: None,
                pause_before_next: false
            })),
            manager: None,
            editor_state: None,
            songs_history: None,
            announcements: vec![],
            mpv_cache:MpvCache::new(),
            unsaved_changes:false
        }
    }

    pub fn save_subtitles(&mut self,wait:bool) {
        match &self.state.read().unwrap().playing_state {
            &PlayingState::Playing(ref video_meta) => {
                if let Some(ref sub) = self.subtitles {
                    let json_file_path = video_meta.json_path();
                    let sub: Subtitles = sub.clone();
                    self.unsaved_changes = false;
                    let thread = ::std::thread::spawn(move || {
                        match ::std::fs::File::create(&json_file_path) {
                            Ok(mut file) => {
                                serde_json::to_writer_pretty(&mut file, &sub).unwrap();
                                info!("Saved file {}", json_file_path.display());
                            },
                            Err(e) => {
                                error!("Failed to write-open subtitles file {} : {:?}",&json_file_path.display(),e);
                            }
                        }
                    });
                    if wait {
                        if let Err(_) = thread.join() {
                            error!("Some error happened in the save json thread");
                        }
                    }
                };
            }
            _ => {
                error!("Could not save file, no video is playing !");
            }
        }
    }

    pub fn start(&mut self, arg_matches: ArgMatches) -> Result<()> {
        let mut is_playlist_empty = true ;
        if let Some(video_files) = arg_matches.values_of("VIDEO_FILE") {
            let mut state = self.state.write().unwrap();
            for value in video_files {
                match VideoMeta::new(value) {
                    Ok(video_meta) => {
                        state.playlist.push_back(video_meta);
                        is_playlist_empty = false;
                    }
                    Err(e) => {
                        error!("Error when importing file : '{}'", e);
                    }
                }
            }
        }
        if arg_matches.is_present("quit") {
            self.state.write().unwrap().quit_when_finished = Some(true);
        } else if arg_matches.is_present("no-quit") {
            self.state.write().unwrap().quit_when_finished = Some(false);
        }
        let mut enable_manager : bool ;
        if arg_matches.is_present("karaoke_mode") {
            self.options.mode = ToyundaMode::KaraokeMode;
            debug!("Enabling karaoke mode");
            enable_manager = true ;
        } else if arg_matches.is_present("edit_mode") {
            self.options.mode = ToyundaMode::EditMode;
            self.editor_state = None;
            if let Err(e) = self.mpv.set_option("loop-file","inf") {
                error!("loop file option failed : {}",e);
            };
            debug!("Enabling edit mode");
            enable_manager = false;
        } else {
            self.options.mode = ToyundaMode::NormalMode;
            enable_manager = true ;
        }
        if let Some(songs_history) = arg_matches.value_of("songs_history") {
            match SongsHistory::new(songs_history) {
                Ok(songs_history) => {
                    self.songs_history = Some(songs_history);
                },
                Err(e) => {
                    error!("songs_history parsing : {}",e);
                }
            }
        };
        if arg_matches.is_present("no_manager") {
            enable_manager = false ;
        }
        if enable_manager {
            let port : String =
                String::from(arg_matches.value_of("manager_port").unwrap_or("8080"));
            let listen_address : String =
                String::from(arg_matches.value_of("manager_listen_address").unwrap_or("0.0.0.0"));
            // list of directories analyzed
            let yaml_dirs = if let Some(yaml_directories) =
                                          arg_matches.values_of("yaml_directory") {
                yaml_directories.map(|v| {
                    PathBuf::from(v)
                }).collect::<Vec<_>>()
            } else {
                vec![]
            };
            let manager = Manager::new(&*format!("{}:{}",listen_address,port), Arc::downgrade(&self.state), yaml_dirs, self.songs_history.as_ref());
            match manager {
                Ok(manager) => {self.manager = Some(manager);},
                Err(e) => {error!("Error when initializing manager : '{}'",e);}
            }
        }

        if let Some(volume) = arg_matches.value_of("volume") {
            match volume.parse::<f64>() {
                Ok(volume) => {
                    match self.mpv.set_property("volume", volume) {
                        Ok(_) => {
                            debug!("Successfully overridden initial volume");
                        }
                        Err(e) => {
                            error!("Could not change initial volume of mpv,\
                                   error '{}' ({:?})", e, e);
                        }
                    };
                }
                Err(e) => {
                    error!("Error when parsing volume param, expected float, got '{}'; (error \
                            {:?})",volume,e);
                }
            }
        }
        if (is_playlist_empty == false) {
            if let Err(e) = self.execute_command(Command::PlayNext) {
                error!("Error trying to play first file : '{}'",e);
            }
        }
        Ok(())
    }

    /// load media actually loads subtitles with it : when the file is finished
    /// being loaded, we can load the subtitles for sure
    ///
    pub fn on_load_media(&mut self) -> Result<ToyundaAction> {
        let res = self.import_cur_file_subtitles();
        if let Err(e) = res {
            if self.options.mode == ToyundaMode::KaraokeMode {
                if let &PlayingState::Playing(ref video_meta) =
                    &self.state.write().unwrap().playing_state {
                        error!("Subtitles import error: {} , file {} will be skipped",
                        e,video_meta.video_path.display());
                    } else {
                        error!("Subtitles import error: {} , file will be skipped",e);
                    }
                    self.execute_command(Command::PlayNext)
            } else {
                error!("Subtitles import error: {}",e);
                Ok(ToyundaAction::Nothing)
            }
        } else {
            Ok(ToyundaAction::Nothing)
        }
    }

    /// This method doesnt load subtitles ... we wait for the file to be loaded
    /// to load subtitles (that way Video-related parameters can be sent to subtitles,
    /// like total length, FPS...
    pub fn load_media_from_video_meta(&mut self,video_meta:VideoMeta)
                                           -> Result<ToyundaAction> {
        let tmp_video_path =
            video_meta.video_path.to_str().map(|s| String::from(s));
        match tmp_video_path {
            None => {
                error!("Invalid UTF-8 Path for {} , skipping file",
                       video_meta.video_path.display());
                self.execute_command(Command::PlayNext)
            }
            Some(video_path) => {
                match self.mpv.command(&["loadfile", video_path.as_str()]) {
                    Ok(_) => {
                        if let Some(ref mut songs_history) = self.songs_history {
                            songs_history.insert_song_history_entry(&*format!("{}",&video_meta));
                        };
                        if let Err(e) = self.displayer.renderer.window_mut().unwrap().set_title(&*format!("Toyunda Player - {}",video_meta)) {
                            warn!("Unexpected error when setting title : {}",e);
                        };
                        self.state.write().unwrap().playing_state =
                            PlayingState::Playing(video_meta);
                        info!("Now playing : '{}'", &video_path);
                        Ok(ToyundaAction::Nothing)
                    },
                    Err(e) => {
                        error!("Trying to play file {} but error {} occured. \
                                Skiping file ...",video_path,e);
                        self.execute_command(Command::PlayNext)
                    },
                }
            }
        }
    }

    pub fn get_file_fps(&self) -> f64 {
        self.mpv.get_property::<f64>("fps").unwrap_or(0.0)
    }

    /// if video_meta is None, reload the current subtitles
    /// otherwise load from video_meta
    pub fn import_cur_file_subtitles(&mut self) -> Result<()> {
        let duration : u32 = 
            (self.mpv.get_property::<f64>("duration").unwrap_or(0.0) * 1000.0) as u32;
        let fps : f64 = self.get_file_fps(); 
        let (json_path, lyr_path, frm_path) =
            match &self.state.read().unwrap().playing_state {
                &PlayingState::Idle => {
                    return Err(Error::Text(String::from("Error when reloading subtitles \
                                                         : no file is playing !")))
                },
                &PlayingState::Playing(ref video_meta) => {
                    (video_meta.json_path(), video_meta.lyr_path(), video_meta.frm_path())
                }
            };
        // TODO move this chunk into a private method
        if (json_path.is_file()) {
            debug!("Loading file {}", json_path.display());
            let json_file = ::std::fs::File::open(json_path).expect("Failed to open JSON file");
            let read_result : ::std::result::Result<Subtitles,_>
                = serde_json::from_reader(json_file);
            match read_result {
                Ok(mut subtitles) => {
                    subtitles.post_init(duration);
                    self.subtitles = Some(subtitles);
                    Ok(())
                },
                Err(e) => {
                    Err(e.into())
                }
            }
        } else {
            debug!("Failed to load json file, trying lyr and frm files");
            if (lyr_path.is_file() && frm_path.is_file()) {
                warn!("Failed to load json subtitle file, loading lyr and frm");
                (Some(&*frm_path), &*lyr_path, fps).into_subtitles()
                    .map(|mut subtitles| {
                        subtitles.post_init(duration);
                        self.subtitles = Some(subtitles);
                        ()
                    })
                    .map_err(|s| Error::Text(s))
            } else if lyr_path.is_file() {
                warn!("Failed to load json subtitle file, loading lyr");
                warn!("Failed to load .frm file; Subtitles won't have timings");
                (None, &*lyr_path, fps).into_subtitles()
                    .map(|mut subtitles| {
                        subtitles.post_init(duration);
                        self.subtitles = Some(subtitles);
                        ()
                    })
                    .map_err(|s| Error::Text(s))
            } else if frm_path.is_file() {
                Err(Error::FileNotFound(frm_path.display().to_string()))
            } else {
                Err(Error::FileNotFound(lyr_path.display().to_string()))
            }
        }
    }
    
    /// this should only be called by "credits_as_overlay_frame"
    // maybe if should be moved somewhere else ? TODO
    fn credits_with_percent_to_overlay(&self,percent:f64) -> OverlayFrame {
        fn inner_percent(global_percent:f64,lower_bound:f64,higher_bound:f64)->f64 {
            (global_percent - lower_bound) / (higher_bound - lower_bound)
        };
        const CREDITS_TRANSITION_TIME : f64 = 0.04;
        let mut overlay_frame = OverlayFrame::new();
        let subs = self.subtitles.as_ref().unwrap();
        // we SHOULD have checked that subtitles arent none beforehand
        if let Some((first_sentence,second_sentence_opt)) = subs.credit_sentences() {
            let mut string = String::new();
            let color : Option<Color> = subs.as_syllable_options(-10000)
                                            .and_then(|o| o.alive_color.map(|c| c.into()));
            let color : Color = color.unwrap_or(Color::new(128,255,128));
            let mut color = AlphaColor::from(color);
            if percent >= 1.0 - CREDITS_TRANSITION_TIME {
                // second credits end transition
                if let Some(second_sentence) = second_sentence_opt {
                    string = second_sentence;
                    color.alpha = (inner_percent(percent,1.0,1.0-CREDITS_TRANSITION_TIME) * 192.0) as u8;
                }
            } else if percent >= 0.5 + CREDITS_TRANSITION_TIME {
                // second credits
                if let Some(second_sentence) = second_sentence_opt {
                    string = second_sentence;
                    color.alpha = 192;
                }
            } else if percent >= 0.5 {
                // second credits begin transition
                if let Some(second_sentence) = second_sentence_opt {
                    string = second_sentence;
                    color.alpha = (inner_percent(percent,0.5,0.5+CREDITS_TRANSITION_TIME) * 192.0) as u8;
                }
            } else if percent >= 0.5 - CREDITS_TRANSITION_TIME {
                // first credits end transition
                string = first_sentence;
                color.alpha = (inner_percent(percent,0.5,0.5-CREDITS_TRANSITION_TIME) * 192.0) as u8;
            } else if percent >= CREDITS_TRANSITION_TIME {
                // first credits
                string = first_sentence;
                color.alpha = 192;
            } else {
                // first credits begin transition
                string = first_sentence;
                color.alpha = (inner_percent(percent,0.0,CREDITS_TRANSITION_TIME) * 192.0) as u8;
            }
            if !string.is_empty() {
                let text_sub_unit = TextSubUnit {
                    text: format!("[{}]",string),
                    attach_logo: false,
                    color: color,
                    outline: Outline::Light(Color::new(0,0,0)),
                    shadow: None,
                };
                overlay_frame.text_units.push(TextUnit {
                    text: vec![text_sub_unit],
                    size: Size::FitPercent(Some(0.96), Some(0.065)),
                    pos: (PosX::FromLeftPercent(0.02),
                          PosY::FromBottomPercent(0.03)),
                    anchor: (0.0, 1.0),
                });
            }
        };
        overlay_frame
    }
    
    pub fn credits_as_overlay_frame(&self) -> OverlayFrame {
        let time_pos = self.get_media_current_time();
        if let Some(ref subtitles) = self.subtitles {
            let opts = &subtitles.subtitles_options; // use as an alias
            let mut percent_credits : f64;
            percent_credits = (time_pos as f64 - opts.start_credits_time as f64) /
                (opts.credits_time as f64);
            if percent_credits >= 0.0 && percent_credits <= 1.0 {
                return self.credits_with_percent_to_overlay(percent_credits);
            } else {
                percent_credits = (time_pos as f64 - opts.end_credits_time as f64) /
                    (opts.credits_time as f64);
                if percent_credits >= 0.0 && percent_credits <= 1.0 {
                    return self.credits_with_percent_to_overlay(percent_credits);
                }
            }
        }
        OverlayFrame::new()
    }

    pub fn messages_as_overlay_frame(&mut self) -> OverlayFrame {
        use chrono::Duration;
        use ::toyunda_player::graphic_message::*;
        let is_karaoke_mode = self.options.mode == ToyundaMode::KaraokeMode;
        let graphic_messages = get_graphic_messages();
        let graphic_messages = graphic_messages.into_iter().filter(|m|{
            !is_karaoke_mode || m.category == Category::Announcement 
        });
        let graphic_messages = 
            graphic_messages.chain(self.announcements.iter().filter_map(|&(ref s,ref t)|{
                if *t + Duration::seconds(8) > Local::now() {
                    Some(GraphicMessage {
                        category: Category::Announcement,
                        text: s.clone()
                    })
                } else {
                    None
                }
            }));
        let message_height = 0.06;
        let mut overlay_frame = OverlayFrame {
            text_units:vec![]
        };
        for (n, message) in graphic_messages.enumerate() {
            let text_elt: TextSubUnit = TextSubUnit {
                text: message.text.clone(),
                attach_logo: false,
                color: match message.category {
                    Category::Error => AlphaColor::new(255, 0, 0),
                    Category::Warn => AlphaColor::new(255, 140, 0),
                    Category::Announcement => AlphaColor::new(255, 255, 255),
                },
                outline: Outline::Light(Color::new(0,0,0)),
                shadow: None,
            };
            let text_unit: TextUnit = TextUnit {
                text: vec![text_elt],
                size: Size::FitPercent(Some(0.98), Some(message_height)),
                pos: (PosX::FromLeftPercent(0.01),
                      PosY::FromBottomPercent(0.01 + message_height * n as f32)),
                anchor: (0.0, 1.0),
            };
            overlay_frame.text_units.push(text_unit);
        }
        overlay_frame
    }

    pub fn render_overlay(&mut self) -> Result<()> {
        use sdl2::rect::Rect;
        let (width, height) = self.displayer.sdl_renderer().window().unwrap().size();
        let time_pos = self.get_media_current_time();
        let display_params : DisplayParams = 
            match (self.mpv_cache.cached_width(),
                   self.mpv_cache.cached_height()) {
                (Some(w),Some(h)) => {
                    let (final_w, final_h);
                    let (offset_x, offset_y);
                    let mpv_aspect_ratio : f64 = (w as f64) / (h as f64) ;
                    let screen_aspect_ratio : f64 = (width as f64)/(height as f64);
                    if mpv_aspect_ratio < screen_aspect_ratio {
                        final_w = ((mpv_aspect_ratio / screen_aspect_ratio)
                                   * (width as f64)) as u32;
                        final_h = height;
                        offset_y = 0;
                        offset_x = ((width - final_w) / 2) as i32;
                    } else {
                        final_h = ((screen_aspect_ratio / mpv_aspect_ratio)
                                   * (height as f64)) as u32;
                        final_w = width;
                        offset_x = 0;
                        offset_y = ((height - final_h) / 2) as i32;
                    };
                    DisplayParams {
                        output_size:Some((final_w,final_h)),
                        offset:Some((offset_x,offset_y))
                    }
                },
                _ => DisplayParams {
                    output_size:None,
                    offset:None
                }
        };
        if let Some(ref subtitles) = self.subtitles {
            if self.state.read().unwrap().display_subtitles {
                let overlay_frame = if let Some(ref editor_state) = self.editor_state {
                    editor_state.to_overlay_frame(time_pos, subtitles)
                } else {
                    subtitles.to_overlay_frame(time_pos)
                };
                match overlay_frame {
                    Ok(overlay_frame) => {
                        self.displayer.display(&overlay_frame,&display_params);
                    },
                    Err(err_string) => {
                        error!("Error when processing overlay : {}",err_string);
                    }
                };
            }
        }
        if (self.options.mode == ToyundaMode::EditMode ) {
            let percent_pos : f64 =
                self.mpv_cache.cached_percent_pos().unwrap_or(0.0);
            let (window_width, window_height) =
                self.displayer.sdl_renderer().window().unwrap().size();
            let rect_width = window_width * 5 / 1000 ;
            let rect_height = rect_width * 2 ;
            let (rect_origin_x, rect_origin_y) =
               ( (((window_width - rect_width) as f64) * percent_pos / 100.0 ) as i32,
                 (window_height - rect_height) as i32) ;
            self.displayer.sdl_renderer_mut().set_draw_color(SdlColor::RGB(0,0,255));
            self.displayer.sdl_renderer_mut()
                          .fill_rect(Rect::new(rect_origin_x,rect_origin_y,
                                               rect_width,rect_height))
                          .unwrap();
        };
        // display logs
        let credits_overlay_frame = self.credits_as_overlay_frame();
        self.displayer.display(&credits_overlay_frame,&display_params);
        let messages_overlay_frame = self.messages_as_overlay_frame();
        self.displayer.display(&messages_overlay_frame,&display_params);
        self.displayer.render();
        Ok(())
    }

    fn get_manager_commands(&mut self) -> Vec<Command> {
        let mut commands: Vec<Command> = Vec::new();
        if let &Some(ref manager) = &self.manager {
            for command in manager.receiver.try_recv() {
                commands.push(command);
            }
        }
        commands
    }

    pub fn on_end_file(&mut self) -> Result<ToyundaAction> {
        self.state.write().unwrap().playing_state = PlayingState::Idle;
        if let Err(e) = self.displayer.renderer.window_mut().unwrap().set_title("Toyunda Player") {
            warn!("Unexpected error when setting title : {}",e);
        };
        self.clear_subtitles();
        self.execute_command(Command::PlayNext)
    }

    /// true : confirm terminate
    /// false : nope
    pub fn on_terminate(&mut self,is_error:bool) -> bool {
        if is_error {
            true
        } else {
            if self.unsaved_changes {
                use sdl2::messagebox::*;
                let buttons : Vec<_> = vec![
                    ButtonData {
                        flags:MESSAGEBOX_BUTTON_RETURNKEY_DEFAULT,
                        button_id:1,
                        text:"Yes"
                    },
                    ButtonData {
                        flags:MESSAGEBOX_BUTTON_NOTHING,
                        button_id:2,
                        text:"No"
                    },
                    ButtonData {
                        flags:MESSAGEBOX_BUTTON_ESCAPEKEY_DEFAULT,
                        button_id:3,
                        text:"Cancel"
                    },
                ];
                let res = show_message_box(MESSAGEBOX_WARNING,
                    buttons.as_slice(), "Save ?",
                    "You are about to discard unsaved changes, save now ?",
                    None, None);
                match res {
                    Ok(ClickedButton::CustomButton(&ButtonData {button_id: 1, ..})) => {
                        self.save_subtitles(true);
                        true
                    },
                    Ok(ClickedButton::CustomButton(&ButtonData {button_id: 2, ..})) => true,
                    Ok(ClickedButton::CustomButton(&ButtonData {button_id: 3, ..})) |
                    Ok(ClickedButton::CloseButton) => false,
                    Ok(ClickedButton::CustomButton(&ButtonData {button_id: i, ..})) => {
                        warn!("Unexpected {} when received answer from message_box",i);
                        false
                    },
                    Err(e) => {
                        error!("Error {:?} when trying to show message_box",e);
                        false
                    }
                }
            } else {
                true
            }
        }
    }

    pub fn main_loop(&mut self, sdl_context: &Sdl) -> Result<()> {
        let mut event_pump = sdl_context.event_pump().expect("Failed to create event_pump");
        // TODO : Add a single queue of `Command` so the result can
        // be processed in the place only.
        let mut command_results : Vec<Result<ToyundaAction>> = Vec::with_capacity(16);
        'main: loop {
            let (width, height) = self.displayer.sdl_renderer().window().unwrap().size();
            let alt_keys = get_alt_keys(event_pump.keyboard_state());
            self.mpv.draw(0, width as i32, -(height as i32)).expect("failed to draw video frame with mpv");
            self.mpv_cache.update(&self.mpv);
            for event in event_pump.poll_iter() {
                command_results.push(self.handle_event(event, alt_keys));
            }
            for command in self.get_manager_commands() {
                command_results.push(self.execute_command(command));
            };
            while let Some(event) = self.mpv.wait_event(0.0) {
                match event {
                    MpvEvent::Shutdown => {break 'main},
                    MpvEvent::EndFile(Ok(MPV_END_FILE_REASON_EOF)) => {
                        match self.on_end_file() {
                            Ok(ToyundaAction::Terminate) => {break 'main},
                            // TODO parse mpv error (shouldnt happen often, but still)
                            _ => {}
                        }
                    },
                    MpvEvent::FileLoaded => {
                        match self.on_load_media() {
                            Ok(_) => {},
                            Err(e) => {
                                error!("{}",e);
                                if self.options.mode == ToyundaMode::KaraokeMode {
                                    command_results.push(self.execute_command(Command::PlayNext));
                                }
                            }
                        }
                    }
                    _ => {}
                };
            }
            for r in command_results.drain(0..) {
                match r {
                    Ok(ToyundaAction::Nothing) => {}
                    Ok(ToyundaAction::Terminate) => {
                        if self.on_terminate(false) {
                            break 'main
                        };
                    },
                    Err(e) => {
                        error!("{}",e);
                    } 
                }
            }
            // TODO change this try into something else,
            // would be bad to crash if we cant render once ...
            try!(self.render_overlay());
        }
        Ok(())
    }

    pub fn start_timing(&mut self,key_id:u8) {
        let time : u32 = self.get_media_current_time();
        if let Some(ref mut editor_state) = self.editor_state {
            if let Some(ref mut subtitles) = self.subtitles {
                self.unsaved_changes = true;
                editor_state.start_timing_syllable(subtitles,time,key_id);
            }
        };
    }

    pub fn end_timing(&mut self,key_id:u8) {
        let time : u32 = self.get_media_current_time();
        if let Some(ref mut editor_state) = self.editor_state {
            if let Some(ref mut subtitles) = self.subtitles {
                self.unsaved_changes = true;
                editor_state.end_timing_syllable(subtitles,time,key_id);
            }
        };
    }

    pub fn get_media_current_time(&self) -> u32 {
        if let Some(t) = self.mpv_cache.cached_time_pos() {
            (t.max(0.0) * 1000.0) as u32
        } else {
            0u32
        }
    }

    pub fn handle_event(&mut self,
                        event: Event,
                        alt_keys_state: (bool, bool, bool))
                        -> Result<ToyundaAction> {
        use ::toyunda_player::ToyundaMode::*;
        let time = self.get_media_current_time();
        let (is_alt_pressed, is_ctrl_pressed, is_shift_pressed) = alt_keys_state;
        let mode = self.options.mode; // shortcut
        match event {
            Event::Quit { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                Ok(ToyundaAction::Terminate),
            Event::KeyDown { keycode: Some(Keycode::S), ..}
                if is_ctrl_pressed && mode == NormalMode => self.execute_command(Command::Stop),
            Event::KeyDown { keycode: Some(Keycode::N), ..}
                if is_ctrl_pressed && mode == NormalMode =>
                    self.execute_command(Command::PlayNext),
            Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. } => {
                self.execute_command(Command::TogglePause)
            }
            Event::KeyDown { keycode: Some(Keycode::E), ..} if mode == EditMode => {
                // toggles editor mode
                if self.editor_state.is_none() {
                    if let Some(subs) = self.subtitles.as_ref() {
                        self.editor_state = Some(EditorState::new(time,subs));
                    };
                } else {
                    self.editor_state = None;
                };
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyDown { keycode: Some(Keycode::J), ..} => {
                let delta = -10i32;
                if let Some(ref mut subtitles) = self.subtitles {
                    if is_alt_pressed && is_shift_pressed {
                        self.unsaved_changes = true;
                        EditorState::shift_subtitles_time(subtitles,delta);
                        return Ok(ToyundaAction::Nothing);
                    };
                    if let Some(ref editor) = self.editor_state {
                        self.unsaved_changes = true;
                        if is_alt_pressed {
                            editor.shift_cur_syllable_end(subtitles,delta);
                        } else if is_shift_pressed {
                            editor.shift_cur_syllable(subtitles,delta);
                        } else {
                            editor.shift_cur_syllable_begin(subtitles,delta);
                        }
                    }
                }
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyDown { keycode: Some(Keycode::K), ..} => {
                let delta = 10i32;
                if let Some(ref mut subtitles) = self.subtitles {
                    if is_alt_pressed && is_shift_pressed {
                        self.unsaved_changes = true;
                        EditorState::shift_subtitles_time(subtitles,delta);
                        return Ok(ToyundaAction::Nothing);
                    };
                    if let Some(ref editor) = self.editor_state {
                        self.unsaved_changes = true;
                        if is_alt_pressed {
                            editor.shift_cur_syllable_end(subtitles,delta);
                        } else if is_shift_pressed {
                            editor.shift_cur_syllable(subtitles,delta);
                        } else {
                            editor.shift_cur_syllable_begin(subtitles,delta);
                        }
                    }
                }
                Ok(ToyundaAction::Nothing)
            },
            // TODO refactor this utter SHIT
            Event::KeyDown {keycode: Some(Keycode::X), repeat:false, .. } => {
                self.start_timing(0);
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyDown {keycode: Some(Keycode::C), repeat:false, .. } => {
                self.start_timing(1);
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyUp {keycode: Some(Keycode::X), .. } => {
                self.end_timing(0);
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyUp {keycode: Some(Keycode::C), .. } => {
                self.end_timing(1);
                Ok(ToyundaAction::Nothing)
            },
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.9))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp9), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.9))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.8))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.8))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.7))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp7), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.7))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.6))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.6))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.5))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp5), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.5))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.4))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.4))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.3))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp3), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.3))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.2))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.2))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(1.1))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp1), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(0.1))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::SetSpeed(2.0))
            }
            Event::KeyDown { keycode: Some(Keycode::Kp0), repeat: false, .. } if mode !=
                                                                                 KaraokeMode => {
                self.execute_command(Command::SetSpeed(1.0))
            }
            Event::KeyDown { keycode: Some(Keycode::KpPlus), .. } => {
                self.execute_command(Command::AddVolume(5))
            }
            Event::KeyDown { keycode: Some(Keycode::KpMinus), .. } => {
                self.execute_command(Command::AddVolume(-5))
            }
            Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. } => {
                if let Some(ref mut editor_state) = self.editor_state {
                    if let Some(ref subs) = self.subtitles {
                        editor_state.prev_sentence(subs);
                    }
                };
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => {
                if let Some(ref mut editor_state) = self.editor_state {
                    if let Some(ref subs) = self.subtitles {
                        editor_state.next_sentence(subs);
                    }
                }
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. }
                if mode != KaraokeMode && is_ctrl_pressed => {
                self.execute_command(Command::Framestep(1))
            }
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::Seek(15.0))
            }
            Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } if mode !=
                                                                                   KaraokeMode => {
                if !is_alt_pressed {
                    if let Some(ref mut editor_state) = self.editor_state {
                        if let Some(ref subs) = self.subtitles {
                            editor_state.next_syllable(subs);
                        };
                        return Ok(ToyundaAction::Nothing);
                    }
                };
                self.execute_command(Command::Seek(3.0))
            }
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. }
                if mode != KaraokeMode && is_ctrl_pressed => {
                self.execute_command(Command::Framestep(-1))
            }
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. }
                if mode != KaraokeMode && is_shift_pressed => {
                self.execute_command(Command::Seek(-15.0))
            }
            Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } if mode !=
                                                                                  KaraokeMode => {
                if !is_alt_pressed {
                    if let Some(ref mut editor_state) = self.editor_state {
                        if let Some(ref subs) = self.subtitles {
                            editor_state.prev_syllable(subs);
                        };
                        return Ok(ToyundaAction::Nothing);
                    }
                };
                self.execute_command(Command::Seek(-3.0))
            }
            Event::KeyDown { keycode: Some(Keycode::R), repeat: false, .. } if mode !=
                                                                               KaraokeMode => {
                self.execute_command(Command::ReloadSubtitles)
            }
            Event::MouseButtonDown {x, y, mouse_btn, .. } if mode != KaraokeMode => {
                let (win_width,win_height) = self.displayer.sdl_renderer().window().unwrap().size();
                if ( y as u32 > win_height * 96 / 100 ) {
                    // bottom 4% : low enough to move
                    let percent_pos : f64 = (100.0 * x as f64 / win_width as f64 ) ;
                    if let Err(e) = self.mpv.set_property("percent-pos",percent_pos) {
                        match e {
                            MPV_ERROR_PROPERTY_UNAVAILABLE => {},
                            // happens when video is paused
                            _ => {
                                error!("Unexpected error: `{}` when trying to move",e);
                            }
                        };
                    }
                } else if mode == EditMode {
                    use sdl2::mouse::Mouse::*;
                    let key_id = match mouse_btn {
                        Left => Some(2),
                        Right => Some(3),
                        _ => None
                    };
                    if let Some(key_id) = key_id {
                        self.start_timing(key_id);
                    };
                };
                Ok(ToyundaAction::Nothing)
            },
            Event::MouseButtonUp {y, mouse_btn, .. } if mode != KaraokeMode => {
                let (_,win_height) = self.displayer.sdl_renderer().window().unwrap().size();
                if ( y as u32 > win_height * 96 / 100 ) {
                    // do nothing
                } else if mode == EditMode {
                    use sdl2::mouse::Mouse::*;
                    let key_id = match mouse_btn {
                        Left => Some(2),
                        Right => Some(3),
                        _ => None
                    };
                    if let Some(key_id) = key_id {
                        self.end_timing(key_id);
                    };
                };
                Ok(ToyundaAction::Nothing)
            },
            Event::DropFile { filename, .. } => {
                match VideoMeta::new(filename) {
                    Ok(video_meta) => self.execute_command(Command::AddToQueue(video_meta)),
                    Err(e) => Err(Error::Text(e)),
                }
            }
            Event::KeyDown { keycode: Some(Keycode::S), repeat: false, .. } if mode !=
                                                                               KaraokeMode => {
                self.save_subtitles(false);
                Ok(ToyundaAction::Nothing)
            }
            Event::KeyDown { keycode: Some(Keycode::V), repeat: false, .. } => {
                self.execute_command(Command::ToggleDisplaySubtitles)
            }
            Event::MouseWheel { y: delta_y, .. } if is_ctrl_pressed => {
                self.execute_command(Command::AddVolume(delta_y as i64))
            }
            Event::MouseWheel { y: delta_y, .. } => {
                self.execute_command(Command::AddVolume(10 * delta_y as i64))
            }
            Event::KeyDown { keycode: Some(Keycode::F), repeat: false, .. } => {
                self.execute_command(Command::ToggleFullscreen)
            }
            _ => Ok(ToyundaAction::Nothing),
        }
    }

    pub fn clear_subtitles(&mut self) {
        self.subtitles = None;
    }
}
