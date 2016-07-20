use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use ::subtitles::sentence::*;
use ::subtitles::syllable::Syllable;
use ::subtitles::metainfo::MetaInfo;
use ::subtitles::options::*;

use ::sdl2::render::Texture;
use ::display::Displayer;

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



    pub fn get_texture_at_frame(&self,displayer:&mut Displayer,frame:u32) -> Result<Texture,String> {
        use sdl2::pixels::Color;
        use sdl2::pixels::PixelFormatEnum::ARGB8888;
        use sdl2::render::TextureValueError;
        fn ceil_power_of_2(v:f64) -> u32 {
            2u32.pow(v.log2().ceil() as u32)
        };
        let (renderer_w,renderer_h) = try!(displayer.sdl_renderer().output_size());
        let texture_width = ceil_power_of_2(renderer_w as f64);
        let texture_height = ceil_power_of_2(renderer_h as f64);
        let mut texture = displayer.sdl_renderer().create_texture_target(ARGB8888,texture_width,texture_height)
            .expect("Failed to create texture");
        texture.set_blend_mode(::sdl2::render::BlendMode::Blend);
        if let Some(ref mut render_target) = displayer.sdl_renderer_mut().render_target() {
            let old_texture = render_target.set(texture).expect("Failed to set texture as target");
            debug_assert!(old_texture.is_none());
        } else {
            error!("Render target are not supported with this GC driver");
            return Err("Render target are not supported with this GC driver".to_string());
            unreachable!()
        };
        {
            displayer.sdl_renderer_mut().set_draw_color(Color::RGBA(0,0,0,0));
            displayer.sdl_renderer_mut().clear();
            // make the texture transparent
            // TODO draw subtitles
        }
        let new_texture = {
            if let Some(ref mut render_target) = displayer.sdl_renderer_mut().render_target() {
                render_target.reset()
                    .expect("Failed to reset render_target")
                    .expect("Failed to retrive texture from renderer")
            } else {
                error!("An unknown error happened; Failed to get render_target from renderer");
                panic!("Failed to get render_target from renderer")
            }
        };
        Ok(new_texture)
    }
}
