use display::*;
use ::subtitles::{Sentence,Subtitles,Syllable,Position as SentencePos};

#[derive(Debug)]
pub enum TextureType {
    Logo,
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

impl Frame {
    pub fn from_subtitles(subtitles:&Subtitles,frame_number:u32) -> Frame {
        let mut frame : Frame = Frame {
            textures:  Vec::with_capacity(4),
            vec_text2d:Vec::with_capacity(4),
        };
        let mut sentence_iter = subtitles.sentences.iter().enumerate().filter(|&(_,ref sentence)| {
            match (sentence.syllables.first(),sentence.syllables.last()) {
                (None,_) | (_,None) => false,
                (Some(ref first_syllable),Some(ref last_syllable)) => {
                    let first_frame = first_syllable.begin
                        .saturating_sub(sentence.sentence_options.transition_time as u32);
                    let last_frame = last_syllable.begin
                        .saturating_add(sentence.sentence_options.transition_time as u32);
                    if ( frame_number >= first_frame && frame_number <= last_frame ) {
                        true
                    } else {
                        false
                    }
                }
            }
        }); // get all the sentences displayed on screen
        for (sentence_number,ref sentence) in sentence_iter {
            let text_pos = match sentence.position {
                SentencePos::Row(l) => {
                    (::display::PosX::Centered,
                     ::display::PosY::FromTopPercent(l as f32 / 10.0 + 0.025 ))
                }
                SentencePos::ForcePos(x,y) => {
                    (::display::PosX::FromLeftPercent(x),
                     ::display::PosY::FromTopPercent(y))
                },
            };
            let text_2d = ::display::Text2D {
                //text: text_elts,
                text:vec![],
                size: ::display::Size::FitPercent(Some(0.95), Some(0.1)),
                pos: text_pos,
                anchor: (0.5, 0.0),
            };
            frame.vec_text2d.push(text_2d);
        };
        frame
    }
}
