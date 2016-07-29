use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use ::subtitles::sentence::*;
use ::subtitles::syllable::Syllable;
use ::subtitles::metainfo::MetaInfo;
use ::subtitles::options::*;
use ::display::*;

use sdl2::render::Texture;
use sdl2::rect::Rect;

#[derive(Debug,Default)]
pub struct Subtitles {
    pub sentences:Vec<Sentence>,
    pub subtitles_options:Option<SubtitlesOptions>,
    pub meta_info:MetaInfo
}

/// subtitles : already stored Subtitles
/// sentence : Sentence to add to the subtitles
fn set_best_sentence_row(sentences:&[Sentence],sentence:&mut Sentence,default_sentence_options:SentenceOptions) {
    let sentence_options : SentenceOptions =
        sentence.sentence_options.unwrap_or(SentenceOptions::default()).or(default_sentence_options);
    let sentence_parameters = SentenceParameters::from(sentence_options);
    // TODO
    let mut best_row = 0u8 ;
    {
        let sentences_candidate = sentences.iter().filter(|sentence_candidate|{
            match (sentence.syllables.first(),sentence.syllables.last(),
                   sentence_candidate.syllables.first(),sentence_candidate.syllables.last()) {
                (None,_,_,_) | (_,None,_,_) | (_,_,None,_) | (_,_,_,None)  => false,
                (Some(ref first_syllable),Some(ref last_syllable),
                 Some(ref first_syllable_candidate),Some(ref last_syllable_candidate)) => {
                    let sentence_candidate_options : SentenceOptions =
                        sentence_candidate.sentence_options.unwrap_or(SentenceOptions::default()).or(default_sentence_options);
                    let sentence_candidate_parameters = SentenceParameters::from(sentence_candidate_options);
                    let first_frame = first_syllable.begin
                        .saturating_sub(sentence_parameters.transition_time as u32);
                    let last_frame = last_syllable.end
                        .saturating_add(sentence_parameters.transition_time as u32);
                    let first_frame_candidate = first_syllable_candidate.begin
                        .saturating_sub(sentence_candidate_parameters.transition_time as u32);
                    let last_frame_candidate = last_syllable_candidate.end
                        .saturating_add(sentence_candidate_parameters.transition_time as u32);
                    if (last_frame_candidate >= first_frame  && last_frame_candidate <= last_frame ) ||
                       (first_frame_candidate >= first_frame && first_frame_candidate <= last_frame) ||
                       (last_frame >= first_frame_candidate  && last_frame <= last_frame_candidate ) ||
                       (first_frame >= first_frame_candidate && first_frame <= last_frame_candidate) {
                        true
                    } else {
                        false
                    }
                }
            }
        });
        let mut taken = vec![];
        for sentence in sentences_candidate {
            match sentence.position {
                Position::Row(i) => {taken.push(i);},
                _ => {}
            }
        };
        while taken.iter().any(|v| *v == best_row) {
            best_row = best_row + 1 ;
        }
    }
    sentence.position = Position::Row(best_row);
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
        let mut current_sentence_options : SentenceOptions = SentenceOptions::default();
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
                                                         syllable_options:None,
                                                     })
                                                     .collect::<Vec<_>>();
                if (lyr_line.starts_with("&")) {
                    syllables.remove(0);
                };
                let sentence = Sentence {
                    syllables : syllables,
                    position : Position::Row(0),
                    sentence_options : Some(current_sentence_options)
                };
                subtitles.sentences.push(sentence);
            } else if lyr_line.starts_with("%color") {
                use utils::parse_bgr;
                let colors : Vec<_> = lyr_line.split_whitespace().collect();
                if colors.len() == 4 {
                    let alive_color = parse_bgr(colors[1]);
                    let transition_color = parse_bgr(colors[2]);
                    let dead_color = parse_bgr(colors[3]);
                    match (alive_color,transition_color,dead_color) {
                        (Ok(alive_color),Ok(transition_color),Ok(dead_color)) => {
                            if current_sentence_options.syllable_options.is_none() {
                                current_sentence_options.syllable_options = Some(SyllableOptions::default());
                            }
                            current_sentence_options.syllable_options.as_mut().unwrap().alive_color = Some(alive_color);
                            current_sentence_options.syllable_options.as_mut().unwrap().transition_color = Some(transition_color);
                            current_sentence_options.syllable_options.as_mut().unwrap().dead_color = Some(dead_color);
                        },
                        _ => {
                            error!("Invalid %color syntax when reading {} at line {} : '{}'",lyr.display(),line_number,lyr_line);
                        }
                    }
                } else {
                    error!("Invalid %color syntax when reading {} at line {} : '{}'",lyr.display(),line_number,lyr_line);
                }
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
        };
        let default_sentence_options : SentenceOptions =
            subtitles.subtitles_options.as_ref().map(
                |ref sub_opts| sub_opts.sentence_options.unwrap_or(SentenceOptions::default())
            ).unwrap_or(SentenceOptions::default());
        for i in 0..subtitles.sentences.len() {
            let (first_half,mut last_half) = subtitles.sentences.split_at_mut(i);
            let sentence = last_half.first_mut().expect("Unexpected None for subtitles last_half");
            set_best_sentence_row(first_half,sentence,default_sentence_options);
        }
        Ok(subtitles)
    }

    /// Note that it will use the render_target used by the renderer,
    /// meaning that it can be a texture or the window depending on what
    /// is used with Displayer
    fn print_subtitle_frame_at(&self,displayer:&mut Displayer,frame_number:u32) -> Result<(),String> {
        let frame = ::display::frame::Frame::from_subtitles(self, frame_number);
        frame.draw(displayer);
        Ok(())
    }

    pub fn get_texture_at_frame(&self,displayer:&mut Displayer,frame:u32) -> Result<Texture,String> {
        use sdl2::pixels::Color;
        use sdl2::pixels::PixelFormatEnum::ARGB8888;
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
        };
        let res = {
            displayer.sdl_renderer_mut().set_viewport(Some(Rect::new(0,0,renderer_w,renderer_h)));
            displayer.sdl_renderer_mut().set_draw_color(Color::RGBA(0,0,0,0));
            // make the texture transparent
            displayer.sdl_renderer_mut().clear();
            // draw subtitles on current render target
            self.print_subtitle_frame_at(displayer,frame)
        };
        displayer.sdl_renderer_mut().set_viewport(None);
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
        res.and(Ok(new_texture))
    }
}
