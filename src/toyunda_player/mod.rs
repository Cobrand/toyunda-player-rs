use mpv::MpvHandlerWithGl;
use ::subtitles::Subtitles;
use ::display::Displayer;
mod options;
pub use self::options::*;

pub struct ToyundaPlayer<'a> {
    subtitles:Option<Subtitles>,
    mpv:Box<MpvHandlerWithGl>,
    displayer:Displayer<'a>,
    options:ToyundaOptions
}
