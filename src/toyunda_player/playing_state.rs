use ::toyunda_player::video_meta::VideoMeta;
#[derive(Debug,Serialize)]
pub enum PlayingState {
    #[serde(rename="idle")]
    Idle,
    #[serde(rename="playing")]
    Playing(VideoMeta)
}
