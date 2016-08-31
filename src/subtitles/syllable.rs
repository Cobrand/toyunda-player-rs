use ::subtitles::options::SyllableOptions;

#[derive(Debug,Default,Serialize,Deserialize,Clone)]
pub struct Syllable {
    pub text: String,
    pub begin: u32,
    pub end: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub syllable_options: Option<SyllableOptions>,
}
