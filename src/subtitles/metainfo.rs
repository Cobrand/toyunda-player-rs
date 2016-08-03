#[derive(Debug,Clone,Serialize,Deserialize)]
#[allow(dead_code)]
pub enum Language {
    Fr,
    En,
    Ger,
    Rus,
    Jp,
    Other(String)
}

#[allow(dead_code)]
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MusicType {
    Opening{number:i32},
    Ending{number:i32},
    Insert,
    Other{
        kind:String,
        #[serde(skip_serializing_if="Option::is_none")]
        number:Option<i32>
    }
}

#[allow(dead_code)]
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MediaType {
    Anime,
    Movie,
    VideoGame,
    Other{
        kind:String
    }
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct MetaInfo {
    pub artist:Option<String>,
    pub origin:Option<String>,// name of anime / movie / video game
    pub song_name:Option<String>,
    pub music_type:Option<MusicType>,
    pub media_type:Option<MediaType>,
    pub language:Option<Language>
}

impl MetaInfo {
    #[allow(dead_code)]
    pub fn new() -> MetaInfo {
        MetaInfo::default()
    }

    #[allow(dead_code)]
    pub fn with_artist(mut self,artist:String) -> MetaInfo {
        self.artist = Some(artist);
        self
    }

    #[allow(dead_code)]
    pub fn with_origin(mut self,origin:String) -> MetaInfo {
        self.origin = Some(origin);
        self
    }

    #[allow(dead_code)]
    pub fn with_song_name(mut self,song_name:String) -> MetaInfo {
        self.song_name = Some(song_name);
        self
    }

    #[allow(dead_code)]
    pub fn with_music_type(mut self,music_type:MusicType) -> MetaInfo {
        self.music_type = Some(music_type);
        self
    }

    #[allow(dead_code)]
    pub fn with_media_type(mut self,media_type:MediaType) -> MetaInfo {
        self.media_type = Some(media_type);
        self
    }

    #[allow(dead_code)]
    pub fn with_language(mut self,language:Language) -> MetaInfo {
        self.language = Some(language);
        self
    }
}
