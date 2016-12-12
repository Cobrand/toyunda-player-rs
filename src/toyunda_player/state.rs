use ::toyunda_player::playlist::Playlist;
use ::toyunda_player::playing_state::PlayingState;


#[derive(Debug,Serialize)]
pub struct State {
    pub playlist: Playlist,
    pub playing_state: PlayingState,
    pub display_subtitles: bool,
    #[serde(skip_serializing_if="Option::is_none")]
    pub quit_when_finished: Option<bool>,
    pub pause_before_next: bool,
}
