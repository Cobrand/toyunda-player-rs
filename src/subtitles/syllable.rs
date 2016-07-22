use ::subtitles::options::SyllableOptions;

#[derive(Debug,Default)]
pub struct Syllable {
    pub text:String,
    pub begin:u32,
    pub end:u32,
    pub syllable_options:SyllableOptions
}
