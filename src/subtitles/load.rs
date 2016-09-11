use super::Subtitles;

pub trait Load {
	fn into_subtitles(&self) -> Result<Subtitles,String>;
}
