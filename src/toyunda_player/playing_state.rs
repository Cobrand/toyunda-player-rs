use std::path::PathBuf;
#[derive(Debug,Clone)]
pub enum PlayingState {
    Idle,
    Playing(PathBuf)
}
