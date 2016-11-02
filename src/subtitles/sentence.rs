use super::{Syllable,SyllableOptions,Subtitles,SubtitlesOptions,AsSyllableOptions};
use super::pos::RowPosition;
use std::ops::Deref;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Sentence {
    pub syllables: Vec<Syllable>,
    #[serde(skip_serializing,skip_deserializing)]
    pub position: RowPosition,
    #[serde(skip_serializing_if="Option::is_none")]
    pub sentence_options: Option<SentenceOptions>,
}

pub trait AsSentenceOptions {
    fn as_sentence_options(&self) -> Option<&SentenceOptions>;
    fn or_sentence_options(&self,other:Option<&SentenceOptions>) -> Option<SentenceOptions> {
        match (self.as_sentence_options(),other) {
            (Some(s),Some(other)) => Some(SentenceOptions {
                transitions: {
                    use std::cmp::Ord;
                    let mut t = s.transitions.iter()
                        .chain(other.transitions.iter())
                        .cloned()
                        .collect::<Vec<SentenceTransition>>();
                    t.sort_by(|a,b| Ord::cmp(&a.offset,&b.offset));
                    t
                },
                syllable_options: s.syllable_options.or_syllable_options(other.syllable_options.as_ref()),
                display_logo: s.display_logo.or(other.display_logo),
                transition_time_after: s.transition_time_after.or(other.transition_time_after),
                fade_time_after: s.fade_time_after.or(other.fade_time_after),
                transition_time_before: s.transition_time_before.or(other.transition_time_before),
                fade_time_before: s.fade_time_before.or(other.fade_time_before),
                row_position: s.row_position.or(other.row_position),
            }),
            (Some(s),None) => Some(s.clone()),
            (None,Some(other)) => Some(other.clone()),
            (None,None) => None
        }
    }
}

impl AsSentenceOptions for Sentence {
    fn as_sentence_options(&self) -> Option<&SentenceOptions> {
        self.sentence_options.as_ref()
    }
}

impl AsSentenceOptions for Subtitles {
    fn as_sentence_options(&self) -> Option<&SentenceOptions> {
        self.subtitles_options.sentence_options.as_ref()
    }
}

impl AsSentenceOptions for SentenceOptions {
    fn as_sentence_options(&self) -> Option<&SentenceOptions> {
        Some(self)
    }
}

impl AsSentenceOptions for SubtitlesOptions {
    fn as_sentence_options(&self) -> Option<&SentenceOptions> {
        self.sentence_options.as_ref()
    }
}

impl<T:AsSentenceOptions> AsSentenceOptions for Option<T> {
    fn as_sentence_options(&self) -> Option<&SentenceOptions> {
        match *self {
            Some(ref t) => t.as_sentence_options(),
            None => None
        }
    }
}

impl Sentence {
    /// used mainly for debugging purposes
    #[allow(dead_code)]
    pub fn text(&self) -> String {
        self.syllables.iter().fold(String::new(),|mut string,syllable| {
            string.push_str(&*syllable.text);
            string
        })
    }
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct SentenceOptions {
    /// Global SyllableOptions
    #[serde(skip_serializing_if="Option::is_none")]
    pub syllable_options: Option<SyllableOptions>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub transitions: Vec<SentenceTransition>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub display_logo: Option<bool>,
    /// Total time where subtitles start appearing, before first syllable start playing
    #[serde(skip_serializing_if="Option::is_none")]
    pub transition_time_before: Option<u16>,
    /// Span where subtitles start appearing
    #[serde(skip_serializing_if="Option::is_none")]
    pub fade_time_before: Option<u16>,
    /// Total time where subtitles are disappearing, before first syllable start playing
    #[serde(skip_serializing_if="Option::is_none")]
    pub transition_time_after: Option<u16>,
    /// Span where subtitles start disappearing
    #[serde(skip_serializing_if="Option::is_none")]
    pub fade_time_after: Option<u16>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub row_position: Option<RowPosition>,
}

impl Deref for SentenceOptions {
    type Target = Option<SyllableOptions>;
    fn deref(&self) -> &Option<SyllableOptions> {
        &self.syllable_options
    }
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct SentenceTransition {
    offset: u32,
    new_options: SentenceOptions
}

#[derive(Debug,Clone)]
pub struct SentenceParameters {
    pub transitions: Vec<SentenceTransition>,
    pub display_logo: bool,
    pub transition_time_before: u16,
    pub fade_time_before: u16,
    pub transition_time_after: u16,
    pub fade_time_after: u16,
    pub row_position: Option<RowPosition>,
}

impl From<SentenceOptions> for SentenceParameters {
    fn from(sentence_options: SentenceOptions) -> Self {
        SentenceParameters {
            transitions: sentence_options.transitions,
            display_logo: sentence_options.display_logo.unwrap_or(true),
            transition_time_before: sentence_options.transition_time_before.unwrap_or(800),
            fade_time_before: sentence_options.fade_time_before.unwrap_or(200),
            transition_time_after: sentence_options.transition_time_after.unwrap_or(500),
            fade_time_after: sentence_options.fade_time_after.unwrap_or(200),
            row_position: sentence_options.row_position,
        }
    }
}

#[test]
fn test_sentence_options_propagation(){
    use super::*;
    use utils::RGB;
    let default_options = SentenceOptions {
        syllable_options: Some(SyllableOptions {
            alive_color: Some(Color::new(128,0,0)),
            dead_color: Some(Color::new(0,128,0)),
            transition_color: None,
            outline: None
        }),
        ..SentenceOptions::default()
    };
    let syllable_options = SyllableOptions {
        alive_color: Some(Color::new(172,172,172)),
        ..SyllableOptions::default()
    };
    let syllable_parameters = SyllableParameters::from(syllable_options.or_syllable_options(default_options.as_syllable_options()).unwrap());
    assert_eq!(syllable_parameters.alive_color,Color::new(172,172,172));
    assert_eq!(syllable_parameters.dead_color,Color::new(0,128,0));
    assert_eq!(syllable_parameters.transition_color,Color::new(255,0,0));
}
