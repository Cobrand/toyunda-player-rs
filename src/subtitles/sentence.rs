use ::subtitles::options::SentenceOptions;
use ::subtitles::syllable::Syllable;
use std::fmt;

#[derive(Debug)]
pub struct Sentence {
    pub syllables:Vec<Syllable>,
    pub position: Position,
    pub sentence_options:SentenceOptions
}

#[derive(Copy,Clone,Debug)]
pub enum Position {
    Row(u8),
    ForcePos(f32,f32)
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
