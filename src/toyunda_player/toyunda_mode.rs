#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum ToyundaMode {
    /// most shortcuts are available
    NormalMode,
    /// shortcuts are different, allow modifying the subtitles directly in the player
    EditMode,
    /// almost no shortcuts are available
    KaraokeMode,
}
