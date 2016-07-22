use ::toyunda_player::*;
use ::toyunda_player::error::{Result,Error};
use std::cmp::{min,max};

#[derive(Copy,Clone,Debug)]
pub enum Command {
    AddVolume(i64),
    SetSpeed(f64),
    Framestep(i32),
    Seek(f64),
    TogglePause,
    ToggleFullscreen,
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
                let (frame_step_type,step) = if step >= 0 {
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
            Command::EndFile => {
                Ok(ToyundaAction::Terminate)
            }
        }
    }
}
