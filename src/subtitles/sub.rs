use ::subtitles::sentence::Sentence;
use std::path::Path;
use ::subtitles::options::SubtitlesOptions;
use ::subtitles::metainfo::MetaInfo;
use ::sdl2::render::{Renderer,Texture};

#[derive(Debug)]
pub struct Subtitles {
    pub sentences:Vec<Sentence>,
    pub subtitles_options:SubtitlesOptions,
    pub meta_info:MetaInfo
}

impl Subtitles {
    pub fn load_from_lyr_frm<P:AsRef<Path>>(lyr:P,frm:P) -> Subtitles {
        unimplemented!()
    }

    pub fn get_texture_at_frame(&self,renderer:Renderer,frame:u32) -> Texture {
        unimplemented!()
    }
}
