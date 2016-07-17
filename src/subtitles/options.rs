use ::sdl2::pixels::Color;

#[derive(Debug,Clone)]
pub struct SubtitlesOptions {
    /// Global SentenceOptions
    pub sentence_options:SentenceOptions,
    /// Total time where subtitles start appearing, before first syllable start playing
    pub transition_time:u16,
    /// Span where subtitles start appearing
    pub fade_time:u16
}

impl Default for SubtitlesOptions {
    fn default() -> SubtitlesOptions {
        SubtitlesOptions {
            sentence_options : SentenceOptions::default(),
            transition_time : 10,
            fade_time : 5
        }
    }
}

#[derive(Debug,Clone)]
pub struct SentenceOptions {
    /// Global SyllableOptions
    pub syllable_options:SyllableOptions,
    pub display_logo:bool
}

impl Default for SentenceOptions {
    fn default() -> SentenceOptions {
        SentenceOptions {
            syllable_options : SyllableOptions::default(),
            display_logo : true
        }
    }
}

#[derive(Debug,Clone)]
pub struct SyllableOptions {
    pub alive_color:Color,
    pub transition_color:Color,
    pub dead_color:Color,
    pub outline:Option<Outline> // color of
}

impl Default for SyllableOptions {
    fn default() -> SyllableOptions {
        SyllableOptions {
            alive_color:      Color::RGB(255,255,  0),
            transition_color: Color::RGB(255,  0,  0),
            dead_color:       Color::RGB(0  ,255,255),
            outline: Some(Outline::default())
        }
    }
}

#[derive(Copy,Clone,Debug)]
pub struct Outline {
    pub color:Color,
    pub ratio:f32
}

impl Default for Outline {
    fn default() -> Outline {
        Outline {color:Color::RGB(0,0,0),ratio:0.05}
    }
}
