use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use ::subtitles::sentence::*;
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

/// subtitles : already stored Subtitles
/// sentence : Sentence to add to the subtitles
fn get_best_sentence_row(subtitles:&Subtitles,sentence:&mut Sentence) {
    // TODO
    unimplemented!()
}

impl Subtitles {
    pub fn load_from_lyr_frm<P:AsRef<Path>>(lyr:P,frm:P) -> Result<Subtitles,String> {
        let frm : &Path = frm.as_ref();
        let lyr : &Path = lyr.as_ref();
        let mut subtitles = Subtitles::default();
        let lyr_file = try!(File::open(lyr).map_err(|e| e.description().to_string() ));
        let frm_file = try!(File::open(frm).map_err(|e| e.description().to_string() ));
        let lyr_file = BufReader::new(&lyr_file);
        let frm_file = BufReader::new(&frm_file);
        for (line_number,lyr_line) in lyr_file.lines().enumerate() {
            let lyr_line = try!(
                lyr_line.map_err(|e|
                    format!("IoError when reading {} at line {} : '{}'",
                            lyr.display(),
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
                let mut row : u8 = 0;
                let sentence = Sentence {
                    syllables : syllables,
                    position : Position::Row(row),
                    sentence_options : SentenceOptions::default()
                };
                subtitles.sentences.push(sentence);
            };
        }
        let mut frames : Vec<(u32,u32)> = vec!();
        for (line_number,frm_line) in frm_file.lines().enumerate() {
            let frm_line = try!(
                frm_line.map_err(|e|
                    format!("IoError when reading {} at line {} : '{}'",
                            frm.display(),
                            line_number,
                            e.description())
                )
            );
            if !frm_line.trim().is_empty() {
                let line_frames : Result<Vec<_>,_> = frm_line.split(' ')
                                                     .map(|s| s.parse::<u32>())
                                                     .collect();
                let begin_end = line_frames.map_err(|e| {
                    format!("{}",e)
                }).and_then(|line_frames| {
                    match (line_frames.get(0),line_frames.get(1),line_frames.get(2)) {
                        (Some(&begin),Some(&end),None) => {
                            Ok((begin,end))
                        },
                        (None,_,_) | (_,None,_) => {
                            Err(format!("Error while parsing frm file '{}' at line {}, not enough values",frm.display(),line_number))
                        },
                        (_,_,Some(_)) => {
                            Err(format!("Error while parsing frm file '{}' at line {}, too many values",frm.display(),line_number))
                        }
                    }
                });
                let begin_end = try!(begin_end);
                frames.push(begin_end);
            } else {
                warn!("empty line {} in frm file '{}'",line_number,frm.display());
            }
        };
        let mut frame_iter = frames.iter();
        'sentences: for sentence in subtitles.sentences.iter_mut() {
            for syllable in sentence.syllables.iter_mut() {
                match frame_iter.next() {
                    Some(&(begin,end)) => {
                        syllable.begin = begin ;
                        syllable.end = end ;
                    },
                    None => {
                        break 'sentences;
                    }
                }
            }
        }
        Ok(subtitles)
    }

    pub fn get_texture_at_frame(&self,renderer:Renderer,frame:u32) -> Texture {
        unimplemented!()
    }
}
