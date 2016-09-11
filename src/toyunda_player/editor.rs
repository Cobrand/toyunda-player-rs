use ::subtitles::{Subtitles,Sentence,Syllable} ;
use ::overlay::{Outline,OverlayFrame,TextUnit,TextSubUnit,AlphaColor,Color};
use ::overlay::pos::*;
use ::utils::{RGBA,RGB};

pub struct EditorState {
    pub current_sentence : u16,
    pub current_syllable : u16,
    /// if this is `Some(t)`, key is being held since
    /// `t`, otherwise it isn't being held.
    pub start_frame_key_1 : Option<u32>,
    pub start_frame_key_2 : Option<u32>
}

impl EditorState {
    pub fn new(time:u32,subs:&Subtitles) -> EditorState {
        let mut i : u16 = 0;
        for (sentence_n,sentence) in subs.sentences.iter().enumerate() {
            if let Some(syll) = sentence.syllables.last() {
                if syll.end.unwrap() > time {
                    i = sentence_n as u16;
                    break;
                }
            }
        };
        EditorState {
            current_sentence:i,
            current_syllable:0,
            start_frame_key_1:None,
            start_frame_key_2:None,
        }
    }

    pub fn holding(&self) -> bool {
        self.start_frame_key_1.is_some() || self.start_frame_key_2.is_some()
    }

    pub fn get_sentence<'a>(&'a self,subs:&'a Subtitles) -> Option<&Sentence> {
       subs.sentences.get(self.current_sentence as usize)
    }

    pub fn get_syllable<'a>(&'a self,subs:&'a Subtitles) -> Option<&Syllable> {
        if let Some(sentence) = self.get_sentence(subs) {
            sentence.syllables.get(self.current_syllable as usize)
        } else {
            None
        }
    }

    pub fn get_sentence_mut<'a>(&'a self,subs:&'a mut Subtitles) -> Option<&mut Sentence> {
       subs.sentences.get_mut(self.current_sentence as usize)
    }

    pub fn get_syllable_mut<'a>(&'a self,subs:&'a mut Subtitles) -> Option<&mut Syllable> {
        if let Some(sentence) = self.get_sentence_mut(subs) {
            sentence.syllables.get_mut(self.current_syllable as usize)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.current_sentence = 0 ;
        self.current_syllable = 0 ;
    }

    pub fn prev_sentence(&mut self, _subs:&Subtitles) {
        if ( self.current_sentence <= 1 ) {
            self.current_sentence = 0 ;
        } else {
            self.current_sentence -= 1 ;
        }
        self.current_syllable = 0 ;
    }

    pub fn next_sentence(&mut self,subs:&Subtitles) {
        if subs.sentences.len() > self.current_sentence as usize + 1 {
            self.current_syllable = 0 ;
            self.current_sentence += 1 ;
        } // Otherwise do nothing (already at max)
    }

    pub fn prev_syllable(&mut self,subs:&Subtitles) {
        if self.get_sentence(subs).is_some() {
            if self.current_syllable == 0 {
                self.prev_sentence(subs);
                self.current_syllable =
                    subs.sentences[self.current_sentence as usize].syllables.len() as u16 - 1;
            } else {
                self.current_syllable -= 1 ;
            }
        } else {
            self.reset();
        }
    }

    pub fn next_syllable(&mut self,subs:&Subtitles) {
        let sentence_len_opt : Option<usize> = self.get_sentence(subs).map(|s| s.syllables.len());
        if let Some(sentence_len) = sentence_len_opt {
            if sentence_len == self.current_syllable as usize + 1 {
                self.next_sentence(subs);
            } else {
                self.current_syllable += 1 ;
            }
        } else {
            self.reset();
        }
    }

    /// time in ms
    pub fn start_timing_syllable(&mut self,subs:&mut Subtitles,time:u32,key:u8) {
        if key == 0 {
            if let Some(_) = self.start_frame_key_2 {
                self.end_timing_syllable(subs,time,1);
            };
            self.start_frame_key_1 = Some(time);
        } else {
            if let Some(_) = self.start_frame_key_1 {
                self.end_timing_syllable(subs,time,0);
            };
            self.start_frame_key_2 = Some(time);
        };
    }

    /// time in ms
    pub fn end_timing_syllable(&mut self,subs:&mut Subtitles,time:u32,key:u8) {
        let b : bool = if let Some(syllable) = self.get_syllable_mut(subs) {
            if key == 0 {
                if let Some(begin_time) = self.start_frame_key_1 {
                    syllable.begin = begin_time ;
                    syllable.end = Some(time) ;
                    true
                } else {
                    false
                }
            } else {
                if let Some(begin_time) = self.start_frame_key_2 {
                    syllable.begin = begin_time ;
                    syllable.end = Some(time) ;
                    true
                } else {
                    false
                }
            }
        } else {
            false
        };
        if b {
            if key == 0 {
                self.start_frame_key_1 = None;
            } else {
                self.start_frame_key_2 = None;
            }
            self.next_syllable(subs);
        };
    }

    pub fn to_overlay_frame(&self,subs:&Subtitles) -> Result<OverlayFrame,String> {
        let mut text_units : Vec<TextUnit> = vec![];
        let cur_syl = self.current_syllable;
        let cur_sen = self.current_sentence;
        let (prev_s,cur_s,next_s) : (Option<&Sentence>,&Sentence,Option<&Sentence>) =
        if cur_sen == 0 {
            if let Some(s) = subs.sentences.get(cur_sen as usize) {
                (None,s,subs.sentences.get(cur_sen as usize + 1))
            } else {
                return Ok(OverlayFrame {
                    text_units:text_units
                });
            }
        } else if cur_sen as usize >= subs.sentences.len() - 1 {
            if let Some(s) = subs.sentences.get(cur_sen as usize) {
                (subs.sentences.get(cur_sen as usize - 1),s,None)
            } else {
                return Ok(OverlayFrame {
                    text_units:text_units
                });
            }
        } else {
            if let Some(s) = subs.sentences.get(cur_sen as usize) {
                (subs.sentences.get(cur_sen as usize - 1),s,subs.sentences.get(cur_sen as usize + 1))
            } else {
                return Ok(OverlayFrame {
                    text_units:text_units
                });
            }
        };
        let outline = Outline::Light(Color::new(0,0,0));
        let text_size = Size::FitPercent(Some(0.95),Some(0.09));
        // let mut cur_sentence_text_elts = vec!();
        if let Some(s) = prev_s {
            let mut syll_text : String = String::new();
            for syll in &s.syllables {
                syll_text.push_str(&*syll.text);
            };
            let text_elts : Vec<TextSubUnit> = vec![
                TextSubUnit {
                    text:syll_text,
                    color: AlphaColor::new_rgba(255,255,255,128),
                    outline:outline,
                    shadow : None,
                    attach_logo : false }
            ];
            text_units.push( TextUnit {
                text:text_elts,
                size: text_size,
                pos : (PosX::Centered,
                       PosY::FromTopPercent(0.05)),
                anchor: (0.5, 0.5)
            });
        }
        if let Some(s) = next_s {
            let mut syll_text : String = String::new();
            for syll in &s.syllables {
                syll_text.push_str(&*syll.text);
            };
            let text_elts : Vec<TextSubUnit> = vec![
                TextSubUnit {
                    text:syll_text,
                    color: AlphaColor::new_rgba(255,255,255,128),
                    outline:outline,
                    shadow : None,
                    attach_logo : false
                }
            ];
            text_units.push( TextUnit {
                text:text_elts,
                size: text_size,
                pos : (PosX::Centered,
                       PosY::FromTopPercent(0.30)),
                anchor: (0.5, 0.5)
            });
        }
        if cur_s.syllables.len() > cur_syl as usize {
            let before = &cur_s.syllables[..(cur_syl as usize)];
            let current = &cur_s.syllables[cur_syl as usize];
            let after = &cur_s.syllables[(cur_syl as usize + 1)..];
            let mut text_elts = vec![];
            if before.len() > 0 {
                text_elts.push(TextSubUnit {
                    text: before.iter().fold(String::new(),|mut string,syllable| {
                        string.push_str(&*syllable.text);
                        string
                    }),
                    color: AlphaColor::new_rgba(255,255,255,255),
                    outline:outline,
                    shadow : None,
                    attach_logo : false
                });
            }
            text_elts.push(TextSubUnit {
                text: current.text.clone(),
                color: if self.holding() {
                    AlphaColor::new_rgba(255,0  ,0,255)
                } else {
                    AlphaColor::new_rgba(255,255,0,255)   
                },
                outline:outline,
                shadow : None,
                attach_logo : true
            });
            if after.len() > 0 {
                text_elts.push(TextSubUnit {
                    text: after.iter().fold(String::new(),|mut string,syllable| {
                        string.push_str(&*syllable.text);
                        string
                    }),
                    color: AlphaColor::new_rgba(255,255,255,255),
                    outline:outline,
                    shadow : None,
                    attach_logo : false
                });
            }
            text_units.push( TextUnit {
                text:text_elts,
                size: text_size,
                pos : (PosX::Centered,
                       PosY::FromTopPercent(0.15)),
                anchor: (0.5, 0.5)
            });
        };
        Ok(OverlayFrame {
            text_units:text_units
        })
    }
}
