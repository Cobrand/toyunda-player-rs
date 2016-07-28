use display::*;
use utils::*;
use ::subtitles::{Sentence,Subtitles,Syllable,Position as SentencePos};
use sdl2::render::TextureQuery;
use sdl2::rect::Rect;

#[derive(Debug)]
#[allow(dead_code)]
pub enum TextureType {
    LyricsLogo,
}

#[derive(Debug)]
pub struct Texture {
    pub texture_type:TextureType,
    pub size: Size,
    pub pos: (PosX, PosY),
    pub anchor: (f32, f32),
}

impl Display for Texture {
    fn draw(self, displayer: &mut Displayer) {
        if displayer.lyrics_logo.is_some() {
            let (window_width, window_height) = displayer.sdl_renderer().window().unwrap().size();
            let (fit_width, fit_height) = match self.size {
                Size::FitPercent(option_x, option_y) => {
                    (option_x.map(|v| (v * window_width as f32) as u32 ),
                     option_y.map(|v| (v * window_height as f32) as u32 ))
                },
                Size::Fit(x, y) => (x, y),
            };
            // TODO refactor position with Text2D
            let TextureQuery { width:original_texture_width, height:original_texture_height, .. } =
                match self.texture_type {
                    TextureType::LyricsLogo => displayer.lyrics_logo.as_ref().unwrap().query(),
            };
            let (final_width,final_height) = match (fit_width,fit_height) {
                (None,None) => {
                    (original_texture_width,original_texture_height)
                },
                (Some(w),None) => {
                    let ratio : f32 = (w as f32 / original_texture_width as f32) ;
                    (
                        w,
                        (ratio * original_texture_height as f32) as u32
                    )
                },
                (None,Some(h)) => {
                    let ratio : f32 = (h as f32 / original_texture_height as f32) ;
                    (
                        (ratio * original_texture_width as f32) as u32,
                        h
                    )
                },
                (Some(w),Some(h)) => {
                    let ratio : f32 =
                       (h as f32 / original_texture_height as f32).min(
                        w as f32 / original_texture_width as f32);
                    (
                        (ratio * original_texture_width as f32) as u32,
                        (ratio * original_texture_height as f32) as u32
                    )
                },
            };
            let (target_pos_x,target_pos_y) = real_position((window_width,window_height),self.pos,self.anchor,(final_width,final_height));
            let target_rect: Rect = Rect::new(target_pos_x, target_pos_y, final_width, final_height);
            match self.texture_type {
                TextureType::LyricsLogo => {
                    displayer.copy_lyrics_logo(target_rect);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub textures:Vec<Texture>,
    pub vec_text2d:Vec<Text2D>
}

impl Display for Frame {
    fn draw(self, displayer: &mut Displayer) {
        for text_2d in self.vec_text2d.into_iter() {
            text_2d.draw(displayer);
        }
        for texture in self.textures.into_iter() {
            texture.draw(displayer);
        }
    }
}

fn compute_sentence_alpha(sentence:&Sentence,frame_number:u32) -> f32 {
    match (sentence.syllables.first(), sentence.syllables.last()) {
        (Some( &Syllable {begin:frame_begin, ..} ),
         Some( &Syllable {end  :frame_end  , ..} )) => {
            let end_fade_frame : u32 = (sentence.sentence_options.transition_time
                                     -  sentence.sentence_options.fade_time) as u32 ;
            let begin_first_fade_frame =
                frame_begin.saturating_sub(sentence.sentence_options.transition_time as u32);
            let end_first_fade_frame =
                frame_begin.saturating_sub(end_fade_frame);
            let begin_second_fade_frame =
                frame_end.saturating_add(end_fade_frame);
            let end_second_fade_frame =
                frame_end.saturating_add(sentence.sentence_options.transition_time as u32);
            debug_assert_eq!(end_second_fade_frame - begin_second_fade_frame,
                             sentence.sentence_options.fade_time as u32);
            if (end_first_fade_frame < frame_number &&
                begin_second_fade_frame > frame_number) {
                1.0
            } else if begin_first_fade_frame <= frame_number &&
               end_first_fade_frame >= frame_number {
                (frame_number - begin_first_fade_frame) as f32 /
                (end_first_fade_frame - begin_first_fade_frame) as f32
            } else if begin_second_fade_frame <= frame_number &&
               end_second_fade_frame >= frame_number {
                1.0 -
                ((frame_number - begin_second_fade_frame) as f32 /
                 (end_second_fade_frame - begin_second_fade_frame) as f32)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn add_syllable(mut text_elts : &mut Vec<::display::TextElement>,
                syllable:&Syllable,
                current_frame:u32,
                alpha:f32) {
    if (current_frame < syllable.begin) {
        let text_2d = ::display::TextElement {
            text: syllable.text.clone(),
            color: fade_color(syllable.syllable_options.alive_color, alpha),
            outline: syllable.syllable_options.outline.map(|outline| outline.color ),
            shadow: None,
            attach_logo:false
        };
        text_elts.push(text_2d);
    } else if (syllable.begin <= current_frame) && (current_frame <= syllable.end) {
        let percent = (current_frame - syllable.begin) as f32 /
                      (syllable.end  - syllable.begin) as f32;
        // lets ease the percent a lil bits
        let percent = 1.0 - (1.0 - percent*percent).sqrt();
        let transition_color = mix_colors(syllable.syllable_options.transition_color,
                                          syllable.syllable_options.dead_color,
                                          percent);
        let text_2d = ::display::TextElement {
            text:  syllable.text.clone(),
            color: transition_color,
            outline: syllable.syllable_options.outline.map(|outline| outline.color ),
            shadow: None,
            attach_logo:false
        };
        text_elts.push(text_2d);
    } else {
        let text_2d = ::display::TextElement {
            text: syllable.text.clone(),
            color: fade_color(syllable.syllable_options.dead_color, alpha),
            outline: syllable.syllable_options.outline.map(|outline| outline.color),
            shadow: None,
            attach_logo:false
        };
        text_elts.push(text_2d);
    }
}

impl Frame {
    pub fn from_subtitles(subtitles:&Subtitles,frame_number:u32) -> Frame {
        let mut frame : Frame = Frame {
            textures:  Vec::with_capacity(0),
            vec_text2d:Vec::with_capacity(4),
        };
        let sentence_iter = subtitles.sentences.iter().enumerate().filter(|&(_,ref sentence)| {
            match (sentence.syllables.first(),sentence.syllables.last()) {
                (None,_) | (_,None) => false,
                (Some(ref first_syllable),Some(ref last_syllable)) => {
                    let first_frame = first_syllable.begin
                        .saturating_sub(sentence.sentence_options.transition_time as u32);
                    let last_frame = last_syllable.end
                        .saturating_add(sentence.sentence_options.transition_time as u32);
                    if ( frame_number >= first_frame && frame_number <= last_frame ) {
                        true
                    } else {
                        false
                    }
                }
            }
        }); // get all the sentences displayed on screen
        for (_sentence_number,ref sentence) in sentence_iter {
            let sentence_alpha = compute_sentence_alpha(sentence,frame_number);
            let mut text_elts = vec![] ;
            let mut logo_position : Option<u16> = None;
            for syllable in sentence.syllables.iter() {
                add_syllable(&mut text_elts,syllable,frame_number,sentence_alpha);
            }
            'syllables: for (n,syllable) in sentence.syllables.iter().enumerate() {
                if frame_number >= syllable.begin {
                    logo_position = Some(n as u16);
                } else {
                    break 'syllables;
                }
            }
            match sentence.syllables.last() {
                Some(ref syllable) => {
                    if (frame_number > syllable.end) {
                        logo_position = None;
                    }
                },
                None => {}
            }
            match logo_position {
                Some(logo_position) => {
                    match text_elts.get_mut(logo_position as usize) {
                        Some(ref mut text_elt) => {
                            text_elt.attach_logo = true ;
                        },
                        None => {
                            error!("Unexpected None in getting from logo_position !")
                        }
                    }
                },
                None => {}
            }
            let text_pos = match sentence.position {
                SentencePos::Row(l) => {
                    (::display::PosX::Centered,
                     ::display::PosY::FromTopPercent( l as f32 * 0.15 + 0.01 ))
                }
                SentencePos::ForcePos(x,y) => {
                    (::display::PosX::FromLeftPercent(x),
                     ::display::PosY::FromTopPercent(y))
                },
            };
            let text_2d = ::display::Text2D {
                text: text_elts,
                size: ::display::Size::FitPercent(Some(0.95), Some(0.09)),
                pos: text_pos,
                anchor: (0.5, 0.0),
            };
            frame.vec_text2d.push(text_2d);
        };
        frame
    }
}
