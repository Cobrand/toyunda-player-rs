use ::subtitles::options::SyllableOptions;

#[derive(Debug,Default,Serialize,Deserialize)]
pub struct Syllable {
    pub text:String,
    pub begin:u32,
    pub end:u32,
    pub syllable_options:Option<SyllableOptions>
}
