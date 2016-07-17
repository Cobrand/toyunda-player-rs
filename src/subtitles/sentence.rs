use ::subtitles::options::SentenceOptions;
use ::subtitles::syllable::Syllable;

#[derive(Debug,Default)]
pub struct Sentence {
    pub syllables:Vec<Syllable>,
    pub sentence_options:SentenceOptions
}
