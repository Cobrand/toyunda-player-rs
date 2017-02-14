use ::toyunda_player::*;
use ::toyunda_player::error::{Result, Error};
use std::cmp::{min, max};
use ::toyunda_player::playing_state::*;
use chrono::{DateTime, Local};

#[derive(Debug)]
pub enum Command {
    AddVolume(i64),
    SetSpeed(f64),
    Framestep(i32),
    Seek(f64),
    TogglePause,
    ToggleFullscreen,
    ToggleDisplaySubtitles,
    ToggleQuitOnFinish,
    PauseBeforeNext,
    AddToQueue(VideoMeta),
    AddToQueueWithPos(VideoMeta, usize),
    DeleteFromQueue(usize),
    ReloadSubtitles,
    /// Stops the queue, but doesnt empty it
    /// Use PlayNext to play the queue again
    Stop,
    PlayNext,
    ClearQueue,
    Quit,
    Announcement(String, DateTime<Local>),
}

impl<'r,'ttf> ToyundaPlayer<'r,'ttf> {
    pub fn execute_command(&mut self, command: Command) -> Result<ToyundaAction> {
        match command {
            Command::SetSpeed(speed) => {
                self.mpv
                    .set_property_async("speed", speed, 1)
                    .map_err(|e| Error::MpvError(e))
                    .map(|_| ToyundaAction::Nothing)
            }
            Command::TogglePause => {
                match self.mpv.get_property("pause") {
                        Ok(true) => self.mpv.set_property_async("pause", false, 1),
                        Ok(false) => self.mpv.set_property_async("pause", true, 1),
                        e @ Err(_) => e.map(|_| ()),
                    }
                    .map_err(|e| Error::MpvError(e))
                    .map(|_| ToyundaAction::Nothing)
            }
            Command::AddVolume(delta) => {
                let max_volume = self.mpv.get_property::<i64>("volume-max");
                let current_volume = self.mpv.get_property::<i64>("volume");
                match (max_volume, current_volume) {
                        (e @ Err(_), _) => e.map(|_| ToyundaAction::Nothing),
                        (_, e @ Err(_)) => e.map(|_| ToyundaAction::Nothing),
                        (Ok(max_volume), Ok(current_volume)) => {
                            let new_volume = min(max(current_volume + delta, 0), max_volume);
                            self.mpv
                                .set_property("volume", new_volume)
                                .map(|_| ToyundaAction::Nothing)
                        }
                    }
                    .map_err(|e| Error::MpvError(e))
            }
            Command::ToggleFullscreen => {
                use sdl2::video::FullscreenType;
                let new_fullscreen_type =
                    match self.displayer.sdl_renderer().window().unwrap().fullscreen_state() {
                        FullscreenType::True | FullscreenType::Desktop => {
                            // TODO warn if 'True'
                            FullscreenType::Off
                        }
                        FullscreenType::Off => FullscreenType::Desktop,
                    };
                self.displayer
                    .sdl_renderer_mut()
                    .window_mut()
                    .unwrap()
                    .set_fullscreen(new_fullscreen_type)
                    .map_err(|e| Error::Text(e))
                    .map(|_| ToyundaAction::Nothing)
            }
            Command::Framestep(step) => {
                let (frame_step_type, _step) = if step >= 0 {
                    ("frame-step", step)
                } else {
                    ("frame-back-step", -step)
                };
                self.mpv
                    .command(&[frame_step_type])
                    .map_err(|e| Error::MpvError(e))
                    .map(|_| ToyundaAction::Nothing)
            }
            Command::Seek(delta) => {
                self.mpv
                    .command(&["seek", delta.to_string().as_str()])
                    .map_err(|e| Error::MpvError(e))
                    .map(|_| ToyundaAction::Nothing)
            }
            Command::ToggleDisplaySubtitles => {
                let current_value = self.state.read().unwrap().display_subtitles;
                self.state.write().unwrap().display_subtitles = !current_value;
                Ok(ToyundaAction::Nothing)
            }
            Command::PlayNext => {
                if self.state.read().unwrap().pause_before_next == true {
                    self.state.write().unwrap().pause_before_next = false;
                    return self.execute_command(Command::Stop);
                }
                let video_meta = self.state.write().unwrap().playlist.pop_front();
                match video_meta {
                    None => {
                        try!(self.execute_command(Command::Stop));
                        match self.state.read().unwrap().quit_when_finished {
                            None => {
                                match self.mode {
                                    ToyundaMode::KaraokeMode => Ok(ToyundaAction::Nothing),
                                    ToyundaMode::NormalMode | ToyundaMode::EditMode => {
                                        Ok(ToyundaAction::Terminate)
                                    }
                                }
                            }
                            Some(b) => {
                                if b {
                                    // "quit_when_finished" override
                                    Ok(ToyundaAction::Terminate)
                                } else {
                                    // "dont_quit_when_finished" override
                                    Ok(ToyundaAction::Nothing)
                                }
                            }
                        }
                    }
                    Some(video_meta) => self.load_media_from_video_meta(video_meta),
                }
            }
            Command::Stop => {
                self.state.write().unwrap().playing_state = PlayingState::Idle;
                if let Err(mpv_err) = self.mpv.command(&["stop"]) {
                    Err(mpv_err.into())
                } else {
                    Ok(ToyundaAction::Nothing)
                }
            }
            Command::ClearQueue => {
                self.state.write().unwrap().playlist.clear();
                Ok(ToyundaAction::Nothing)
            }
            Command::AddToQueue(video_meta) => {
                self.state.write().unwrap().playlist.push_back(video_meta);
                Ok(ToyundaAction::Nothing)
            }
            Command::AddToQueueWithPos(video_meta, pos) => {
                if let Ok(mut state) = self.state.write() {
                    if state.playlist.len() >= pos {
                        state.playlist.insert(pos, video_meta);
                        Ok(ToyundaAction::Nothing)
                    } else {
                        Err(format!("Trying to insert element at {} while playlist is of size {}",
                                    pos,
                                    state.playlist.len())
                            .into())
                    }
                } else {
                    Err(format!("Unable to get state object, is the lock poisoned ?").into())
                }
            }
            Command::ReloadSubtitles => {
                try!(self.import_cur_file_subtitles());
                Ok(ToyundaAction::Nothing)
            }
            Command::Quit => Ok(ToyundaAction::Terminate),
            Command::PauseBeforeNext => {
                self.state.write().unwrap().pause_before_next = true;
                Ok(ToyundaAction::Nothing)
            }
            Command::ToggleQuitOnFinish => {
                let b: bool = self.state.read().unwrap().quit_when_finished.unwrap_or(false);
                self.state.write().unwrap().quit_when_finished = Some(!b);
                Ok(ToyundaAction::Nothing)
            }
            Command::Announcement(text, datetime) => {
                self.announcements.push((text, datetime));
                Ok(ToyundaAction::Nothing)
            }
            Command::DeleteFromQueue(p) => {
                if let Ok(mut state) = self.state.write() {
                    if p < state.playlist.len() {
                        state.playlist.remove(p);
                    } else {
                        warn!("Trying to remove {}th element from playlist, but is out of bounds",
                              p);
                    }
                }
                Ok(ToyundaAction::Nothing)
            }
        }
    }
}
