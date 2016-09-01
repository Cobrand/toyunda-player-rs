use ::toyunda_player::playlist::Playlist;
use ::toyunda_player::playing_state::PlayingState;


#[derive(Debug,Serialize)]
pub struct State {
    pub playlist: Playlist,
    pub playing_state: PlayingState
}
