use super::song_info::SongInfo;
use super::pos::{RowPosition, Point};
use super::{Sentence, SentenceOptions, SentenceParameters, Syllable, SyllableOptions,
            SyllableParameters, AsSentenceOptions, AsSyllableOptions};
use ::overlay::*;
use ::overlay::pos::*;
use std::ops::Deref;
use ::utils::*;

#[derive(Debug,Default,Serialize,Deserialize,Clone)]
pub struct Subtitles {
    pub sentences: Vec<Sentence>,
    #[serde(default)]
    pub subtitles_options: SubtitlesOptions,
    #[serde(default)]
    pub song_info: SongInfo,
}

/// subtitles : already stored Subtitles
/// sentence : Sentence to add to the subtitles
fn set_best_sentence_row(sentences: (&[Sentence], &[Sentence]),
                         sentence: &mut Sentence,
                         default_sentence_options: Option<&SentenceOptions>) {
    if let Some(row_pos) = sentence.sentence_options.as_ref().and_then(|o| o.row_position) {
        sentence.position = row_pos;
        return; // life is meaningless
    }
    let (before_sentences, after_sentences) = sentences;
    let sentence_options: Option<SentenceOptions> = sentence.sentence_options
        .or_sentence_options(default_sentence_options);
    let sentence_parameters =
        SentenceParameters::from((sentence_options.unwrap_or(SentenceOptions::default()),
                                  -10000i32));
    let mut best_row = 0u8;
    {
        let filter_fun = |sentence_candidate: &&Sentence| {
            match (sentence.syllables.first(),
                   sentence.syllables.last(),
                   sentence_candidate.syllables.first(),
                   sentence_candidate.syllables.last()) {
                (None, _, _, _) | (_, None, _, _) | (_, _, None, _) | (_, _, _, None) => false,
                (Some(ref first_syllable),
                 Some(ref last_syllable),
                 Some(ref first_syllable_candidate),
                 Some(ref last_syllable_candidate)) => {
                    let sentence_candidate_options: Option<SentenceOptions> =
                        sentence_candidate.sentence_options
                            .or_sentence_options(default_sentence_options);
                    let sentence_candidate_parameters =
                        SentenceParameters::from(
                            (sentence_candidate_options.unwrap_or(SentenceOptions::default()),-10000i32));
                    let first_frame = first_syllable.begin
                        .saturating_sub(sentence_parameters.transition_time_before as u32);
                    let last_frame = last_syllable.end
                        .expect("last syllable has no end")
                        .saturating_add(sentence_parameters.transition_time_after as u32);
                    let first_frame_candidate = first_syllable_candidate.begin
                        .saturating_sub(sentence_candidate_parameters.transition_time_before as u32);
                    let last_frame_candidate = last_syllable_candidate.end
                        .expect("last syllable has no end")
                        .saturating_add(sentence_candidate_parameters.transition_time_after as u32);
                    if (last_frame_candidate >= first_frame &&
                        last_frame_candidate <= last_frame) ||
                       (first_frame_candidate >= first_frame &&
                        first_frame_candidate <= last_frame) ||
                       (last_frame >= first_frame_candidate &&
                        last_frame <= last_frame_candidate) ||
                       (first_frame >= first_frame_candidate &&
                        first_frame <= last_frame_candidate) {
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
        let sentences_candidate_after = after_sentences.iter()
            .filter(|s| {
                if let Some(ref sentence_options) = s.sentence_options {
                    sentence_options.row_position.is_some()
                } else {
                    false
                }
            })
            .filter(&filter_fun);
        let mut taken = vec![];
        for sentence in sentences_candidate_before {
            match sentence.position {
                RowPosition::Row(i) => {
                    taken.push(i);
                }
                _ => {}
            }
        }
        for sentence in sentences_candidate_after {
            if let &RowPosition::Row(i) = sentence.sentence_options
                .as_ref()
                .unwrap()
                .row_position
                .as_ref()
                .unwrap() {
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
    pub fn credit_sentences(&self) -> Option<(String, Option<String>)> {
        self.song_info.credit_sentences()
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

    /// length in ms
    pub fn post_init(&mut self, duration: u32) {
        self.adjust_sentences_row();
        let mut options = &mut self.subtitles_options;
        let credits_time = 8000;
        if (options.credits_time == 0) {
            options.credits_time = credits_time;
        }
        if (options.start_credits_time == 0) {
            options.start_credits_time = 4000;
        }
        if (options.end_credits_time == 0) {
            options.end_credits_time = duration.saturating_sub(4000 + credits_time);
        }
    }

    fn adjust_sentences_row(&mut self) {
        for i in 0..self.sentences.len() {
            let (first_half, mut last_half) = self.sentences.split_at_mut(i);
            let (mut middle, last_half) = last_half.split_first_mut().unwrap();
            set_best_sentence_row((first_half, last_half),
                                  middle,
                                  self.subtitles_options.as_sentence_options());
        }
    }

    // TODO create a  subtitles::Error type and replace String with this
    pub fn to_overlay_frame(&self, current_time: u32) -> Result<OverlayFrame, String> {
        let mut text_units: Vec<TextUnit> = vec![];
        let default_sentence_options: Option<&SentenceOptions> = self.subtitles_options
            .as_sentence_options();
        let sentence_iter = self.sentences.iter().enumerate().filter(|&(_, ref sentence)| {
            let sentence_options: Option<SentenceOptions> = sentence.sentence_options
                .or_sentence_options(default_sentence_options);
            let sentence_parameters =
                SentenceParameters::from((sentence_options.unwrap_or(SentenceOptions::default()),
                                          current_time as i32 -
                                          sentence.syllables.first().unwrap().begin as i32));
            match (sentence.syllables.first(), sentence.syllables.last()) {
                (None, _) | (_, None) => false,
                (Some(&Syllable { begin: first_syllable_begin, .. }),
                 Some(&Syllable { end: Some(last_syllable_end), .. })) => {
                    let first_frame = first_syllable_begin
                        .saturating_sub(sentence_parameters.transition_time_before as u32);
                    let last_frame = last_syllable_end
                        .saturating_add(sentence_parameters.transition_time_after as u32);
                    if (current_time >= first_frame && current_time <= last_frame) {
                        true
                    } else {
                        false
                    }
                }
                _ => panic!("Subtitles have not been checked"),
            }
        }); // get all the sentences displayed on screen
        for (_sentence_number, ref sentence) in sentence_iter {
            let cur_offset: i32 = current_time as i32 -
                                  sentence.syllables.first().unwrap().begin as i32;
            let sentence_alpha =
                compute_sentence_alpha(sentence, default_sentence_options, current_time);
            let mut text_elts = vec![];
            let mut logo_position: Option<u16> = None;
            let sentence_options: Option<SentenceOptions> =
                sentence.or_sentence_options(default_sentence_options);
            let default_syllable_options: Option<SyllableOptions> =
                sentence_options.as_syllable_options(cur_offset);
            let sentence_params =
                SentenceParameters::from((sentence_options.unwrap_or(Default::default()),
                                          cur_offset));
            {
                for tmp_syllables in sentence.syllables.windows(2) {
                    let (syllable1, syllable2) = (&tmp_syllables[0], &tmp_syllables[1]);
                    if !syllable1.text.is_empty() {
                        add_syllable(&mut text_elts,
                                     syllable1,
                                     Some(syllable2),
                                     default_syllable_options.as_ref(),
                                     current_time,
                                     sentence_alpha);
                    }
                }
                match sentence.syllables.last() {
                    Some(last_syllable) => {
                        add_syllable(&mut text_elts,
                                     last_syllable,
                                     None,
                                     default_syllable_options.as_ref(),
                                     current_time,
                                     sentence_alpha);
                    }
                    _ => {}
                }
            }
            if sentence_params.display_logo {
                'syllables: for (n, syllable) in sentence.syllables.iter().enumerate() {
                    if current_time >= syllable.begin {
                        logo_position = Some(n as u16);
                    } else {
                        break 'syllables;
                    }
                }
                match sentence.syllables.last() {
                    Some(ref syllable) => {
                        if (current_time > syllable.end.unwrap()) {
                            logo_position = None;
                        }
                    }
                    None => {}
                }
                match logo_position {
                    Some(logo_position) => {
                        match text_elts.get_mut(logo_position as usize) {
                            Some(ref mut text_elt) => {
                                text_elt.attach_logo = true;
                            }
                            None => error!("Unexpected None in getting from logo_position !"),
                        }
                    }
                    None => {}
                }
            }
            let text_pos = match sentence_params.row_position.unwrap_or(sentence.position) {
                RowPosition::Row(l) => {
                    (PosX::Centered, PosY::FromTopPercent(l as f32 * 0.15 + 0.01))
                }
                RowPosition::ForcePos(Point { x, y }) => {
                    (PosX::FromLeftPercent(x), PosY::FromTopPercent(y))
                }
            };
            let text_unit = TextUnit {
                text: text_elts,
                size: Size::FitPercent(sentence_params.size.width, sentence_params.size.height),
                pos: text_pos,
                anchor: (0.5, 0.0),
            };
            text_units.push(text_unit);
        }
        Ok(OverlayFrame { text_units: text_units })
    }
}

impl Deref for SubtitlesOptions {
    type Target = Option<SentenceOptions>;
    fn deref(&self) -> &Option<SentenceOptions> {
        &self.sentence_options
    }
}

#[derive(Debug,Default,Clone,Serialize,Deserialize)]
pub struct SubtitlesOptions {
    /// Global SentenceOptions
    pub sentence_options: Option<SentenceOptions>,
    #[serde(default)]
    pub start_credits_time: u32,
    #[serde(default)]
    pub end_credits_time: u32,
    #[serde(default)]
    pub credits_time: u32,
}

fn add_syllable(mut text_subunits: &mut Vec<TextSubUnit>,
                syllable: &Syllable,
                next_syllable: Option<&Syllable>,
                default_syllable_options: Option<&SyllableOptions>,
                current_frame: u32,
                alpha: f32) {
    let syllable_end = syllable.end
        .or(next_syllable.map(|s| s.begin.saturating_sub(1)))
        .expect("File has not been checked properly : end syllable has no end frame");
    let syllable_options = syllable.syllable_options.or_syllable_options(default_syllable_options);
    let syllable_parameters =
        SyllableParameters::from(syllable_options.unwrap_or(SyllableOptions::default()));
    let outline = Outline::from(syllable_parameters.outline);
    let alive_color = AlphaColor::from(Color::from(syllable_parameters.alive_color));
    let transition_color = Color::from(syllable_parameters.transition_color);
    let dead_color = Color::from(syllable_parameters.dead_color);
    if (current_frame < syllable.begin) {
        let text_sub_unit = TextSubUnit {
            text: syllable.text.clone(),
            color: fade_color(alive_color, alpha),
            outline: outline,
            shadow: None,
            attach_logo: false,
        };
        text_subunits.push(text_sub_unit);
    } else if (syllable.begin <= current_frame) && (current_frame <= syllable_end) {
        let percent = (current_frame - syllable.begin) as f32 /
                      (syllable_end - syllable.begin) as f32;
        // lets ease the percent a lil bits
        let percent = 1.0 - (1.0 - percent * percent).sqrt();
        let transition_color = AlphaColor::from(mix_colors(transition_color, dead_color, percent));
        let text_sub_unit = TextSubUnit {
            text: syllable.text.clone(),
            color: transition_color,
            outline: outline,
            shadow: None,
            attach_logo: false,
        };
        text_subunits.push(text_sub_unit);
    } else {
        let text_sub_unit = TextSubUnit {
            text: syllable.text.clone(),
            color: fade_color(AlphaColor::from(dead_color), alpha),
            outline: outline,
            shadow: None,
            attach_logo: false,
        };
        text_subunits.push(text_sub_unit);
    }
}

fn compute_sentence_alpha(sentence: &Sentence,
                          default_sentence_options: Option<&SentenceOptions>,
                          frame_number: u32)
                          -> f32 {
    let sentence_options: Option<SentenceOptions> = sentence.sentence_options
        .or_sentence_options(default_sentence_options);
    let sentence_parameters =
        SentenceParameters::from((sentence_options.unwrap_or(SentenceOptions::default()),
                                  frame_number as i32 -
                                  sentence.syllables.first().unwrap().begin as i32));
    match (sentence.syllables.first(), sentence.syllables.last()) {
        (Some(&Syllable { begin: frame_begin, .. }),
         Some(&Syllable { end: Some(frame_end), .. })) => {
            let end_fade_frame_before: u32 =
                (sentence_parameters.transition_time_before -
                 sentence_parameters.fade_time_before) as u32;
            let end_fade_frame_after: u32 =
                (sentence_parameters.transition_time_after -
                 sentence_parameters.fade_time_after) as u32;
            let begin_first_fade_frame =
                frame_begin.saturating_sub(sentence_parameters.transition_time_before as u32);
            let end_first_fade_frame = frame_begin.saturating_sub(end_fade_frame_before);
            let begin_second_fade_frame = frame_end.saturating_add(end_fade_frame_after);
            let end_second_fade_frame =
                frame_end.saturating_add(sentence_parameters.transition_time_after as u32);
            debug_assert_eq!(end_second_fade_frame - begin_second_fade_frame,
                             sentence_parameters.fade_time_after as u32);
            if (end_first_fade_frame < frame_number && begin_second_fade_frame > frame_number) {
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
