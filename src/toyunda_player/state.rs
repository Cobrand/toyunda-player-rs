use ::toyunda_player::playlist::Playlist;
use ::toyunda_player::playing_state::PlayingState;


pub struct State {
	pub playlist : Playlist,
	pub playing_state : PlayingState,
	pub logs : Vec<String>
}
