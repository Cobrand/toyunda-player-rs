use ::toyunda_player::playlist::VideoMeta;
#[derive(Debug)]
pub enum PlayingState {
    Idle,
    Playing(VideoMeta)
}
