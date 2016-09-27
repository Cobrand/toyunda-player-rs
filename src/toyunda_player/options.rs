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
}

impl Default for ToyundaOptions {
    fn default() -> ToyundaOptions {
        ToyundaOptions {
            mode: ToyundaMode::NormalMode,
        }
    }
}
