use std::path::PathBuf;
pub enum PlayingState {
    Idle,
    Playing(PathBuf)
}
