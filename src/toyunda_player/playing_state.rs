use ::toyunda_player::video_meta::VideoMeta;
#[derive(Debug,Serialize)]
pub enum PlayingState {
    #[serde(rename="idle")]
    Idle,
    #[serde(rename="playing")]
    Playing(VideoMeta),
}

impl PlayingState {
    pub fn stop(self) -> (PlayingState, Option<VideoMeta>) {
        match self {
            PlayingState::Idle => (PlayingState::Idle, None),
            PlayingState::Playing(video_meta) => (PlayingState::Idle, Some(video_meta)),
        }
    }
}
