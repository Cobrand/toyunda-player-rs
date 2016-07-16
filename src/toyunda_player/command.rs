use ::toyunda_player::*;
use ::toyunda_player::error::Result;

pub enum Command {
    AddVolume(i32),
    SetSpeed(f64),
    Pause,
    EndFile,
}

impl<'a> ToyundaPlayer<'a> {
    pub fn execute_command(&mut self,comand:Command) -> Result<()> {
        Ok(())
    }
}
