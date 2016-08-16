use ::toyunda_player::playlist::VideoMeta;
#[derive(Debug,Serialize)]
pub enum PlayingState {
    #[serde(rename="idle")]
    Idle,
    #[serde(rename="playing")]
    Playing(VideoMeta)
}
