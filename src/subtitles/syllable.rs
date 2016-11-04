use super::*;

#[derive(Debug,Default,Serialize,Deserialize,Clone)]
pub struct Syllable {
    pub text: String,
    /// time in ms ; floats have the risk of rounding wrong
    pub begin: u32,
    /// Optional End Syllable
    #[serde(skip_serializing_if="Option::is_none")]
    pub end: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub syllable_options: Option<SyllableOptions>,
}

#[derive(Debug,Clone,Copy,Default,Serialize,Deserialize)]
pub struct SyllableOptions {
    #[serde(skip_serializing_if="Option::is_none")]
    pub alive_color: Option<Color>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub transition_color: Option<Color>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub dead_color: Option<Color>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub outline: Option<Outline>,
}

pub trait AsSyllableOptions {
    fn as_syllable_options(&self) -> Option<&SyllableOptions>;
    fn or_syllable_options(&self,other:Option<&SyllableOptions>) -> Option<SyllableOptions> {
        match (self.as_syllable_options(),other) {
            (Some(s),Some(other)) => Some(SyllableOptions {
                alive_color: s.alive_color.or(other.alive_color),
                transition_color: s.transition_color.or(other.transition_color),
                dead_color: s.dead_color.or(other.dead_color),
                outline: s.outline.or(other.outline),
            }),
            (Some(s),None) => Some(s.clone()),
            (None,Some(other)) => Some(other.clone()),
            (None,None) => None
        }
    }
}

// impl AsSyllableOptions for Subtitles{
//     fn as_syllable_options(&self) -> Option<&SyllableOptions> {
//         self.subtitles_options.as_syllable_options()
//     }
// }

// impl AsSyllableOptions for SubtitlesOptions {
//     fn as_syllable_options(&self) -> Option<&SyllableOptions> {
//         if let Some(ref sen_options) = self.sentence_options {
//             sen_options.as_syllable_options()
//         } else {
//             None
//         }
//     }
// }

// impl AsSyllableOptions for Sentence {
//     fn as_syllable_options(&self) -> Option<&SyllableOptions> {
//         if let Some(ref sen_options) = self.sentence_options {
//             sen_options.as_syllable_options()
//         } else {
//             None
//         }
//     }
// }

// impl AsSyllableOptions for SentenceOptions {
//     fn as_syllable_options(&self) -> Option<&SyllableOptions> {
//         self.syllable_options.as_ref()
//     }
// }

impl AsSyllableOptions for Syllable {
    fn as_syllable_options(&self) -> Option<&SyllableOptions> {
        self.syllable_options.as_ref()
    }
}

impl AsSyllableOptions for SyllableOptions {
    fn as_syllable_options(&self) -> Option<&SyllableOptions> {
        Some(self)
    }
}

impl<T:AsSyllableOptions> AsSyllableOptions for Option<T> {
    fn as_syllable_options(&self) -> Option<&SyllableOptions> {
        match *self {
            Some(ref t) => t.as_syllable_options(),
            None => None
        }
    }
}

#[derive(Debug,Clone)]
pub struct SyllableParameters {
    pub alive_color: Color,
    pub transition_color: Color,
    pub dead_color: Color,
    pub outline: Outline,
}

impl From<SyllableOptions> for SyllableParameters {
    fn from(syllable_options: SyllableOptions) -> Self {
        SyllableParameters {
            alive_color: syllable_options.alive_color
                .unwrap_or(Color {red: 255,green: 255,blue:   0}),
            transition_color: syllable_options.transition_color
                .unwrap_or(Color {red: 255,green:   0,blue:   0}),
            dead_color: syllable_options.dead_color
                .unwrap_or(Color {red: 0  ,green: 255,blue: 255}),
            outline: syllable_options.outline
                .unwrap_or(Outline {
                    color: Color {red: 0  ,green: 0  ,blue: 0  },
                    size: 1
                }),
        }
    }
}
