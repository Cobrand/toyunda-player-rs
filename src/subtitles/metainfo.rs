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

#[derive(Debug,Clone,Default)]
pub struct MetaInfo {
    pub artist:Option<String>,
    pub origin:Option<String>,// name of anime / movie / video game
    pub song_name:Option<String>,
    pub music_type:Option<MusicType>,
    pub media_type:Option<MediaType>,
    pub language:Option<Language>
}

impl MetaInfo {
    pub fn new() -> MetaInfo {
        MetaInfo::default()
    }

    pub fn with_artist(mut self,artist:String) -> MetaInfo {
        self.artist = Some(artist);
        self
    }

    pub fn with_origin(mut self,origin:String) -> MetaInfo {
        self.origin = Some(origin);
        self
    }

    pub fn with_song_name(mut self,song_name:String) -> MetaInfo {
        self.song_name = Some(song_name);
        self
    }

    pub fn with_music_type(mut self,music_type:MusicType) -> MetaInfo {
        self.music_type = Some(music_type);
        self
    }

    pub fn with_media_type(mut self,media_type:MediaType) -> MetaInfo {
        self.media_type = Some(media_type);
        self
    }

    pub fn with_language(mut self,language:Language) -> MetaInfo {
        self.language = Some(language);
        self
    }
}
