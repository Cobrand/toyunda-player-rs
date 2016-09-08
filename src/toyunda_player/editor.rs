use ::subtitles::* ;

pub struct EditorState {
    pub current_sentence : u16,
    pub current_syllable : u16,
    /// if this is `Some(t)`, key is being held since
    /// `t`, otherwise it isn't being held.
    pub start_frame_key_1 : Option<u32>,
    pub start_frame_key_2 : Option<u32>
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            current_sentence:0,
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

    pub fn start_timing_syllable(&mut self,subs:&mut Subtitles,frame:u32,key:u8) {
        if key == 0 {
            if let Some(_) = self.start_frame_key_2 {
                self.end_timing_syllable(subs,frame,1);
            };
            self.start_frame_key_1 = Some(frame);
        } else {
            if let Some(_) = self.start_frame_key_1 {
                self.end_timing_syllable(subs,frame,0);
            };
            self.start_frame_key_2 = Some(frame);
        };
    }

    pub fn end_timing_syllable(&mut self,subs:&mut Subtitles,frame:u32,key:u8) {
        let b : bool = if let Some(syllable) = self.get_syllable_mut(subs) {
            if key == 0 {
                if let Some(begin_frame) = self.start_frame_key_1 {
                    syllable.begin = begin_frame ;
                    syllable.end = Some(frame) ;
                    true
                } else {
                    false
                }
            } else {
                if let Some(begin_frame) = self.start_frame_key_2 {
                    syllable.begin = begin_frame ;
                    syllable.end = Some(frame) ;
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
}
