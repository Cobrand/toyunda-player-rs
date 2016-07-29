use ::sdl2::pixels::Color;
use std::convert::From;

#[derive(Debug,Clone)]
pub struct SubtitlesOptions {
    /// Global SentenceOptions
    pub sentence_options:Option<SentenceOptions>,
}

impl Default for SubtitlesOptions {
    fn default() -> SubtitlesOptions {
        SubtitlesOptions {
            sentence_options : None,
        }
    }
}

impl SubtitlesOptions {
    pub fn or(&self,other : SubtitlesOptions) -> SubtitlesOptions {
        SubtitlesOptions {
            sentence_options:self.sentence_options.or(other.sentence_options)
        }
    }
}

#[derive(Debug,Clone,Copy,Default)]
pub struct SentenceOptions {
    /// Global SyllableOptions
    pub syllable_options:Option<SyllableOptions>,
    pub display_logo:Option<bool>,
    /// Total time where subtitles start appearing, before first syllable start playing
    pub transition_time:Option<u16>,
    /// Span where subtitles start appearing
    pub fade_time:Option<u16>
}

impl SentenceOptions {
    pub fn or(&self,other : SentenceOptions) -> SentenceOptions {
        SentenceOptions {
            syllable_options:self.syllable_options.or(other.syllable_options),
            display_logo:self.display_logo.or(other.display_logo),
            transition_time:self.transition_time.or(other.transition_time),
            fade_time:self.fade_time.or(other.fade_time),
        }
    }
}

#[derive(Debug,Clone)]
pub struct SentenceParameters {
    pub display_logo:bool,
    pub transition_time:u16,
    pub fade_time:u16
}

impl From<SentenceOptions> for SentenceParameters {
    fn from(sentence_options:SentenceOptions) -> Self {
        SentenceParameters {
            display_logo: sentence_options.display_logo.unwrap_or(true),
            transition_time: sentence_options.transition_time.unwrap_or(12),
            fade_time: sentence_options.fade_time.unwrap_or(8),
        }
    }
}

#[derive(Debug,Clone,Copy,Default)]
pub struct SyllableOptions {
    pub alive_color:Option<Color>,
    pub transition_color:Option<Color>,
    pub dead_color:Option<Color>,
    pub outline:Option<Option<Color>> // color of
}

impl SyllableOptions {
    pub fn or(&self,other:SyllableOptions) -> SyllableOptions {
        SyllableOptions {
            alive_color:self.alive_color.or(other.alive_color),
            transition_color:self.transition_color.or(other.transition_color),
            dead_color:self.dead_color.or(other.dead_color),
            outline:self.outline.or(other.outline),
        }
    }
}

#[derive(Debug,Clone)]
pub struct SyllableParameters {
    pub alive_color:Color,
    pub transition_color:Color,
    pub dead_color:Color,
    pub outline:Option<Color>
}

impl From<SyllableOptions> for SyllableParameters {
    fn from(syllable_options:SyllableOptions) -> Self {
        SyllableParameters {
            alive_color:      syllable_options.alive_color.     unwrap_or(Color::RGB(255,255,  0)),
            transition_color: syllable_options.transition_color.unwrap_or(Color::RGB(255,  0,  0)),
            dead_color:       syllable_options.dead_color.      unwrap_or(Color::RGB(  0,255,255)),
            outline: syllable_options.outline.unwrap_or(Some(Color::RGB(0,0,0))),
        }
    }
}
