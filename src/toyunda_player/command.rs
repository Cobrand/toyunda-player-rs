use ::toyunda_player::*;
use ::toyunda_player::error::{Result,Error};
use std::cmp::{min,max};
use std::path::PathBuf;
use ::toyunda_player::playing_state::*;

#[derive(Debug)]
pub enum Command {
    AddVolume(i64),
    SetSpeed(f64),
    Framestep(i32),
    Seek(f64),
    TogglePause,
    ToggleFullscreen,
    ToggleDisplaySubtitles,
    AddToQueue(PathBuf),
    ReloadSubtitles,
    PlayNext,
    ClearQueue,
    EndFile,
}

impl<'a> ToyundaPlayer<'a> {
    pub fn execute_command(&mut self,command:Command) -> Result<ToyundaAction> {
        match command {
            Command::SetSpeed(speed) => {
                self.mpv_mut().set_property_async("speed", speed, 1)
                              .map_err(|e| Error::MpvError(e))
                              .map(|_| ToyundaAction::Nothing)
            },
            Command::TogglePause => {
                match self.mpv().get_property("pause") {
                    Ok(true) => {
                        self.mpv_mut().set_property_async("pause", false, 1)

                    }
                    Ok(false) => {
                        self.mpv_mut().set_property_async("pause", true, 1)
                    },
                    e @ Err(_) => e.map(|_| ())
                }
                .map_err(|e| Error::MpvError(e))
                .map(|_| ToyundaAction::Nothing)
            }
            Command::AddVolume(delta) => {
                let max_volume = self.mpv().get_property::<i64>("volume-max");
                let current_volume= self.mpv().get_property::<i64>("volume");
                match (max_volume,current_volume) {
                    (e @ Err(_),_) => e.map(|_| ToyundaAction::Nothing),
                    (_,e @ Err(_)) => e.map(|_| ToyundaAction::Nothing),
                    (Ok(max_volume),Ok(current_volume)) => {
                        let new_volume = min(max(current_volume + delta, 0), max_volume);
                        self.mpv_mut().set_property("volume", new_volume)
                            .map(|_| ToyundaAction::Nothing)
                    }
                }
                .map_err(|e| Error::MpvError(e))
            }
            Command::ToggleFullscreen => {
                use sdl2::video::FullscreenType;
                let new_fullscreen_type =
                    match self.displayer().sdl_renderer().window().unwrap().fullscreen_state() {
                        FullscreenType::True | FullscreenType::Desktop => { // TODO warn if 'True'
                            FullscreenType::Off
                        },
                        FullscreenType::Off => {
                            FullscreenType::Desktop
                        }
                    };
                self.displayer_mut()
                    .sdl_renderer_mut()
                    .window_mut()
                    .unwrap()
                    .set_fullscreen(new_fullscreen_type)
                    .map_err(|e| Error::Text(e))
                    .map(|_| ToyundaAction::Nothing)
            }
            Command::Framestep(step) => {
                let (frame_step_type,_step) = if step >= 0 {
                    ("frame-step",step)
                }  else {
                    ("frame-back-step",-step)
                };
                self.mpv_mut()
                    .command(&[frame_step_type])
                    .map_err(|e| Error::MpvError(e))
                    .map(|_| ToyundaAction::Nothing)
            },
            Command::Seek(delta) => {
                self.mpv_mut()
                    .command(&["seek",delta.to_string().as_str()])
                    .map_err(|e| Error::MpvError(e))
                    .map(|_| ToyundaAction::Nothing)
            },
            Command::ToggleDisplaySubtitles => {
                let current_value = self.options().display_subtitles;
                self.options_mut().display_subtitles = !current_value ;
                Ok(ToyundaAction::Nothing)
            },
            Command::PlayNext => {
                let video_meta = self.state().write().unwrap().playlist.pop_front();
                // match self.mpv_mut().command(&["stop"]) {
                //     Err(e) => {error!("Unexpected error {} ({:?}) happened when stopping player",e,e)},
                //     _ => {}
                // }; // skips 2 videos at once if this is enabled ; TODO : investigate sherlock
                match video_meta {
                    None => {
                        match self.options().quit_when_finished {
                            None => {
                                match self.options().mode {
                                    ToyundaMode::KaraokeMode | ToyundaMode::EditMode =>
                                        Ok(ToyundaAction::Nothing),
                                    ToyundaMode::NormalMode =>
                                        Ok(ToyundaAction::Terminate)
                                }
                            },
                            Some(b) => {
                                if b { // "quit_when_finished" override
                                    Ok(ToyundaAction::Terminate)
                                } else { // "dont_quit_when_finished" override
                                    Ok(ToyundaAction::Nothing)
                                }
                            }
                        }
                    },
                    Some(video_meta) => {
                        let tmp_video_path = video_meta.video_path.to_str().map(|s| {
                            String::from(s)
                        });
                        match tmp_video_path {
                            None => {
                                error!("Invalid UTF-8 Path for {} , skipping file",video_meta.video_path.display());
                                Ok(ToyundaAction::PlayNext)
                            },
                            Some(video_path) => {
                                match self.mpv_mut().command(&["loadfile",video_path.as_str()]) {
                                    Ok(_) => {
                                        match self.import_subtitles(Some(&video_meta)) {
                                            Ok(_) => {
                                                info!("Now playing : '{}'",&video_path);
                                                self.state().write().unwrap().playing_state = PlayingState::Playing(video_meta);
                                                Ok(ToyundaAction::Nothing)
                                            },
                                            Err(e) => {
                                                if self.options().mode == ToyundaMode::KaraokeMode {
                                                    error!("Error was received when importing subtitles : {} , file {} will be skipped",e,video_path);
                                                    Ok(ToyundaAction::PlayNext)
                                                } else {
                                                    let string = format!("Error was received when importing subtitles : {}",e);
                                                    error!("{}",string.as_str());
                                                    self.add_graphic_message(graphic_message::Category::Error, string.as_str());
                                                    Ok(ToyundaAction::Nothing)
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        error!("Trying to play file {} but error {} occured. Skiping file ...",video_path,e);
                                        Ok(ToyundaAction::PlayNext)
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Command::ClearQueue => {
                self.state().write().unwrap().playlist.clear();
                Ok(ToyundaAction::Nothing)
            },
            Command::AddToQueue(path) => {
                match VideoMeta::new(&path) {
                    Ok(video_meta) => {
                        self.state().write().unwrap().playlist.push_back(video_meta);
                    },
                    Err(e) => {
                        error!("Error when adding '{}' to queue : {}",path.display(),e);
                        self.add_graphic_message(
                            graphic_message::Category::Error,
                            &*format!("Error when adding '{}' to queue : {}",path.display(),e)
                        );
                    }
                }
                Ok(ToyundaAction::Nothing)
            }
            Command::EndFile => {
                self.state().write().unwrap().playing_state = PlayingState::Idle;
                self.clear_subtitles();
                Ok(ToyundaAction::Nothing)
            }
            Command::ReloadSubtitles => {
                try!(self.reload_subtitles());
                Ok(ToyundaAction::Nothing)
            }
        }
    }
}
