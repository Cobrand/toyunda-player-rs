use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use ::subtitles::sentence::Sentence;
use ::subtitles::syllable::Syllable;
use ::subtitles::metainfo::MetaInfo;
use ::subtitles::options::*;

use ::sdl2::render::{Renderer,Texture};

#[derive(Debug,Default)]
pub struct Subtitles {
    pub sentences:Vec<Sentence>,
    pub subtitles_options:SubtitlesOptions,
    pub meta_info:MetaInfo
}

impl Subtitles {
    pub fn load_from_lyr_frm<P:AsRef<Path>>(lyr:P,frm:P) -> Result<Subtitles,String> {
        let mut subtitles = Subtitles::default();
        let lyr_file = try!(File::open(lyr).map_err(|e| e.description().to_string() ));
        let frm_file = try!(File::open(frm).map_err(|e| e.description().to_string() ));
        let lyr_file = BufReader::new(&lyr_file);
        let frm_file = BufReader::new(&frm_file);
        for (line_number,lyr_line) in lyr_file.lines().enumerate() {
            let lyr_line = try!(
                lyr_line.map_err(|e|
                    format!("IoError when reading file at line {} : '{}'",
                            line_number,
                            e.description())
                )
            );
            if (!lyr_line.starts_with("%") && !lyr_line.is_empty()) {
                let mut syllables : Vec<_> = lyr_line.split('&')
                                                     .map(|s|
                                                     Syllable {
                                                         text:s.to_string(),
                                                         begin:0,
                                                         end:0,
                                                         syllable_options:SyllableOptions::default()
                                                     })
                                                     .collect::<Vec<_>>();
                if (lyr_line.starts_with("&")) {
                    syllables.remove(0);
                };
                let sentence = Sentence {
                    syllables : syllables,
                    sentence_options : SentenceOptions::default()
                };
                subtitles.sentences.push(sentence);
            };
        }
        Ok(subtitles)
    }

    pub fn get_texture_at_frame(&self,renderer:Renderer,frame:u32) -> Texture {
        unimplemented!()
    }
}
