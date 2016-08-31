#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum ToyundaMode {
    /// most shortcuts are available
    NormalMode,
    /// shortcuts are different, allow modifying the subtitles idrectly in the player
    EditMode,
    /// almost no shortcuts are available
    KaraokeMode,
}

pub struct ToyundaOptions {
    pub mode: ToyundaMode,
    pub display_subtitles: bool,
    pub quit_when_finished: Option<bool>,
}

impl Default for ToyundaOptions {
    fn default() -> ToyundaOptions {
        ToyundaOptions {
            mode: ToyundaMode::NormalMode,
            display_subtitles: true,
            quit_when_finished: None,
        }
    }
}
