use super::{Syllable,SyllableOptions};
use super::pos::RowPosition;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Sentence {
    pub syllables: Vec<Syllable>,
    #[serde(skip_serializing,skip_deserializing)]
    pub position: RowPosition,
    #[serde(skip_serializing_if="Option::is_none")]
    pub sentence_options: Option<SentenceOptions>,
}

impl Sentence {
    pub fn text(&self) -> String {
        self.syllables.iter().fold(String::new(),|mut string,syllable| {
            string.push_str(&*syllable.text);
            string
        })
    }
}

#[derive(Debug,Clone,Copy,Default,Serialize,Deserialize)]
pub struct SentenceOptions {
    /// Global SyllableOptions
    #[serde(skip_serializing_if="Option::is_none")]
    pub syllable_options: Option<SyllableOptions>,
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

impl SentenceOptions {
    pub fn or(&self, other: &SentenceOptions) -> SentenceOptions {
        SentenceOptions {
            syllable_options: self.syllable_options.or(other.syllable_options),
            display_logo: self.display_logo.or(other.display_logo),
            transition_time_after: self.transition_time_after.or(other.transition_time_after),
            fade_time_after: self.fade_time_after.or(other.fade_time_after),
            transition_time_before: self.transition_time_before.or(other.transition_time_before),
            fade_time_before: self.fade_time_before.or(other.fade_time_before),
            row_position: self.row_position.or(other.row_position),
        }
    }
}

#[derive(Debug,Clone)]
pub struct SentenceParameters {
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
            display_logo: sentence_options.display_logo.unwrap_or(true),
            transition_time_before: sentence_options.transition_time_before.unwrap_or(18),
            fade_time_before: sentence_options.fade_time_before.unwrap_or(8),
            transition_time_after: sentence_options.transition_time_after.unwrap_or(12),
            fade_time_after: sentence_options.fade_time_after.unwrap_or(8),
            row_position: sentence_options.row_position,
        }
    }
}
