use ::subtitles::options::SentenceOptions;
use ::subtitles::syllable::Syllable;

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
