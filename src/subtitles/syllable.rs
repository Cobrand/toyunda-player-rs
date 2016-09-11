use super::misc::*;

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

impl SyllableOptions {
    pub fn or(&self, other: &SyllableOptions) -> SyllableOptions {
        SyllableOptions {
            alive_color: self.alive_color.or(other.alive_color),
            transition_color: self.transition_color.or(other.transition_color),
            dead_color: self.dead_color.or(other.dead_color),
            outline: self.outline.or(other.outline),
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
                    color :Color {red: 0  ,green: 0  ,blue: 0  }
                }),
        }
    }
}
