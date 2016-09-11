use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::error::Error;

use super::song_info::SongInfo;
use super::pos::*;
use super::{Sentence,SentenceOptions,SentenceParameters,
        Syllable,SyllableOptions,SyllableParameters};
use ::overlay::*;

use sdl2::render::Texture;

#[derive(Debug,Default,Serialize,Deserialize,Clone)]
pub struct Subtitles {
    pub sentences: Vec<Sentence>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub subtitles_options: Option<SubtitlesOptions>,
    pub song_info: Option<SongInfo>,
}

/// subtitles : already stored Subtitles
/// sentence : Sentence to add to the subtitles
fn set_best_sentence_row(sentences: (&[Sentence],&[Sentence]),
                         sentence: &mut Sentence,
                         default_sentence_options: &SentenceOptions) {
    if let Some(row_pos) = sentence.sentence_options.and_then(|o|o.row_position) {
        sentence.position = row_pos ;
        return; // life is meaningless
    }
    let (before_sentences,after_sentences) = sentences ;
    let sentence_options: SentenceOptions = sentence.sentence_options
        .unwrap_or(SentenceOptions::default())
        .or(default_sentence_options);
    let sentence_parameters = SentenceParameters::from(sentence_options);
    let mut best_row = 0u8;
    {
        let filter_fun = |sentence_candidate:&&Sentence|{
            match (sentence.syllables.first(),sentence.syllables.last(),
                   sentence_candidate.syllables.first(),sentence_candidate.syllables.last()) {
                (None,_,_,_) | (_,None,_,_) | (_,_,None,_) | (_,_,_,None)  => false,
                (Some(ref first_syllable),Some(ref last_syllable),
                 Some(ref first_syllable_candidate),Some(ref last_syllable_candidate)) => {
                    let sentence_candidate_options : SentenceOptions =
                        sentence_candidate.sentence_options.unwrap_or(SentenceOptions::default()).or(&default_sentence_options);
                    let sentence_candidate_parameters = SentenceParameters::from(sentence_candidate_options);
                    let first_frame = first_syllable.begin
                        .saturating_sub(sentence_parameters.transition_time_before as u32);
                    let last_frame = last_syllable.end.expect("last syllable has no end")
                        .saturating_add(sentence_parameters.transition_time_after as u32);
                    let first_frame_candidate = first_syllable_candidate.begin
                        .saturating_sub(sentence_candidate_parameters.transition_time_before as u32);
                    let last_frame_candidate = last_syllable_candidate.end.expect("last syllable has no end")
                        .saturating_add(sentence_candidate_parameters.transition_time_after as u32);
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
        };
        // TODO remove unwraps
        // step 1 : filter -> filter_map to remove "options" and maybe convert directly in
        // RowPosition::Row(_)
        let sentences_candidate_before = before_sentences.iter().filter(&filter_fun);
        let sentences_candidate_after = after_sentences.iter().filter(|s|{
            if let Some(sentence_options) = s.sentence_options {
                sentence_options.row_position.is_some()
            } else {
                false
            }
        }).filter(&filter_fun);
        let mut taken = vec![];
        for sentence in sentences_candidate_before {
            match sentence.position {
                RowPosition::Row(i) => {
                    taken.push(i);
                },
                _ => {}
            }
        };
        for sentence in sentences_candidate_after {
            if let RowPosition::Row(i) = sentence.sentence_options.unwrap().row_position.unwrap() {
                taken.push(i);
            };
        }
        while taken.iter().any(|v| *v == best_row) {
            best_row = best_row + 1;
        }
    }
    sentence.position = RowPosition::Row(best_row);
}

impl Subtitles {
    pub fn credit_sentences(&self) -> Option<(String,Option<String>)> {
        self.song_info.as_ref().and_then(|song_info|{
            song_info.credit_sentences()
        })
    }

    pub fn check(&self) -> Result<(), String> {
        for (s_number, sentence) in self.sentences.iter().enumerate() {
            match (sentence.syllables.first(), sentence.syllables.last()) {
                (Some(_), Some(&Syllable { end: Some(_), .. })) => {}
                (Some(_), Some(&Syllable { end: None, .. })) => {
                    return Err(format!("Error at sentence {}, no 'end' time for the last syllable",
                                       s_number))
                }
                _ => {
                    warn!("Empty sentence {} when checking", s_number);
                }
            };
        }
        Ok(())
    }

    pub fn post_init(&mut self) {
        self.adjust_sentences_row();
    }

    fn adjust_sentences_row(&mut self) {
        let default_sentence_options: SentenceOptions = self.subtitles_options
            .as_ref()
            .map(|ref sub_opts| sub_opts.sentence_options.unwrap_or(SentenceOptions::default()))
            .unwrap_or(SentenceOptions::default());
        for i in 0..self.sentences.len() {
            let (first_half, mut last_half) = self.sentences.split_at_mut(i);
            let (mut middle, last_half) = last_half.split_first_mut().unwrap();
            set_best_sentence_row((first_half,last_half), middle,&default_sentence_options);
        }
    }

    // TODO create a  subtitles::Error type and replace String with this
    pub fn to_overlay_frame(&self,time:u32) -> Result<OverlayFrame,String> {
        let mut text_units : Vec<TextUnit> = vec![];
        Ok(OverlayFrame {
            text_units:text_units
        })
    }
}

#[derive(Debug,Default,Clone,Copy,Serialize,Deserialize)]
pub struct SubtitlesOptions {
    /// Global SentenceOptions
    pub sentence_options: Option<SentenceOptions>,
}
