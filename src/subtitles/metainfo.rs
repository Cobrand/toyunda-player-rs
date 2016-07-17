#[derive(Debug,Clone)]
pub enum Language {
    Fr,
    En,
    Ger,
    Rus,
    Jp,
    Other(String)
}

#[derive(Debug,Clone)]
pub enum MusicType {
    Opening(i32),
    Ending(i32),
    Insert,
    Other(String)
}

#[derive(Debug,Clone)]
pub enum MediaType {
    Anime,
    Movie,
    VideoGame,
    Other(String)
}

#[derive(Debug,Clone)]
pub struct MetaInfo {
    pub artist:String,
    pub origin:String,// name of anime / movie / video game
    pub music_type:MusicType,
    pub media_type:MediaType,
    pub language:Language
}
