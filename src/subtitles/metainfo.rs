#[derive(Debug,Clone,Serialize,Deserialize)]
#[allow(dead_code)]
pub enum Language {
    Fr,
    En,
    Ger,
    Rus,
    Jp,
    Other(String),
}

#[allow(dead_code)]
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MusicType {
    Opening,
    Ending,
    Insert,
    Other(String),
}

impl MusicType {
    pub fn short(&self) -> &str {
        match *self {
            MusicType::Opening => "OP",
            MusicType::Ending => "ED",
            MusicType::Insert => "INS",
            MusicType::Other(ref s) => s.as_str()
        }
    }

    pub fn long(&self) -> &str {
        match *self {
            MusicType::Opening => "Opening",
            MusicType::Ending => "Ending",
            MusicType::Insert => "Insert",
            MusicType::Other(ref s) => s.as_str()
        }
    }
}

#[allow(dead_code)]
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MediaType {
    Anime,
    Movie,
    VideoGame,
    AMV,
    Other(String),
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct MetaInfo {
    #[serde(skip_serializing_if="Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    /// name of anime / movie / video game
    pub media_title: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub media_alt_titles: Option<Vec<String>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub song_name: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub music_type: Option<MusicType>,
    #[serde(skip_serializing_if="Option::is_none")]
    /// 5 of ED5 for instance
    pub music_number: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    /// "v2" of ED5v2
    pub version: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    /// Anime / Movie / ...
    pub media_type: Option<MediaType>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub language: Option<Language>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub creditless: Option<bool>,
}
