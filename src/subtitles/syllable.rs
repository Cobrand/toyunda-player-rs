use ::subtitles::options::SyllableOptions;

#[derive(Debug)]
pub struct Syllable {
    text:String,
    begin:u32,
    end:u32,
    syllable_options:SyllableOptions
}
