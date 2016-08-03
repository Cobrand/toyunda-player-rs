use ::subtitles::options::SentenceOptions;
use ::subtitles::syllable::Syllable;
use std::fmt;

#[derive(Debug,Serialize,Deserialize)]
pub struct Sentence {
    pub syllables:Vec<Syllable>,
    pub position: Position,
    pub sentence_options:Option<SentenceOptions>
}

#[derive(Copy,Clone,Debug,Serialize,Deserialize)]
pub enum Position {
    Row(u8),
    #[allow(dead_code)]
    ForcePos {
        x:f32,
        y:f32
    }
}

impl fmt::Display for Sentence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = self.syllables.iter().fold(String::new(),|mut string,syllable|{
            string.push_str(&*syllable.text);
            string
        });
        write!(f, "\"{}\"", string)
    }
}
