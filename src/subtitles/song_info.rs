extern crate serde;
use serde::{Serialize,Deserialize,Serializer,Deserializer};

impl Serialize for Language {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(),S::Error>
        where S: Serializer
    {
        serializer.serialize_str(match *self {
            Language::Fr => "FR",
            Language::En => "EN",
            Language::Jp => "JAP",
            Language::Ger => "GER",
            Language::Rus => "RUS",
            Language::Instrumental => "INSTRUMENTAL",
            Language::Other(ref string) => string.as_str()
        })
    }
}

impl Deserialize for Language {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct Visitor;
        impl ::serde::de::Visitor for Visitor {
            type Value = Language;
            fn visit_str<E>(&mut self, value: &str) -> Result<Language,E>
                where E: ::serde::de::Error
            {
                Ok(match value {
                    "JAP" | "Jp" | "JP" | "jp" | "jap" => Language::Jp,
                    "FRA" | "Fr" | "FR" | "fr" | "fra" => Language::Fr,
                    "GER" | "Ger" | "ger" => Language::Ger,
                    "RUS" | "Rus" | "rus" => Language::Rus,
                    "EN" | "ENG" | "En" | "en" => Language::En,
                    "INSTRUMENTAL" => Language::Instrumental,
                    s => Language::Other(String::from(s))
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for MusicType {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(),S::Error>
        where S: Serializer
    {
        serializer.serialize_str(match *self {
            MusicType::AMV => "AMV",
            MusicType::Opening => "OP",
            MusicType::Ending => "ED",
            MusicType::Insert => "INS",
            /// Original Sound Track
            MusicType::OST => "OST",
            MusicType::Other(ref string) => string.as_str(),
        })
    }
}

impl Deserialize for MusicType {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct Visitor;
        impl ::serde::de::Visitor for Visitor {
            type Value = MusicType;
            fn visit_str<E>(&mut self, value: &str) -> Result<MusicType,E>
                where E: ::serde::de::Error
            {
                Ok(match value {
                    "AMV" | "Amv" | "amv" => MusicType::AMV,
                    "OP" | "OPENING" | "Op" | "Opening" => MusicType::Opening,
                    "ED" | "ENDING" | "Ed" | "Ending" => MusicType::Ending,
                    "INS" | "INSERT" | "Insert" => MusicType::Insert,
                    "OST" | "Ost" => MusicType::OST,
                    s => MusicType::Other(String::from(s))
                })
            }
        }
        
        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug,Clone)]
pub enum Language {
    Fr,
    En,
    Ger,
    Rus,
    Jp,
    /// Instrumental : why would you play this in the first place ?
    Instrumental,
    Other(String),
}


#[derive(Debug,Clone)]
pub enum MusicType {
    AMV,
    Opening,
    Ending,
    Insert,
    /// Original Sound Track
    OST,
    Other(String),
}

impl MusicType {
    pub fn short(&self) -> &str {
        match *self {
            MusicType::AMV => "AMV",
            MusicType::Opening => "OP",
            MusicType::Ending => "ED",
            MusicType::Insert => "INS",
            MusicType::OST => "OST",
            MusicType::Other(ref s) => s.as_str()
        }
    }

    pub fn long(&self) -> &str {
        match *self {
            MusicType::AMV => "AMV",
            MusicType::Opening => "Opening",
            MusicType::Ending => "Ending",
            MusicType::Insert => "Insert",
            MusicType::OST => "Original Soundtrack",
            MusicType::Other(ref s) => s.as_str()
        }
    }
}

/// Case of an AMV, please tell the source of the visual material
#[allow(dead_code)]
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MediaType {
    Anime,
    Movie,
    VideoGame,
    /// music doesn't come from anything,
    /// it's justa music by itself (e.g. Vocaloid stuff)
    Original,
    Other(String),
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct SongInfo {
    pub artist: Option<String>,
    /// name of anime / movie / video game
    pub media_title: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub media_alt_titles: Option<Vec<String>>,
    pub song_name: Option<String>,
    pub music_type: Option<MusicType>,
    /// 5 of ED5 for instance
    pub music_number: Option<u32>,
    #[serde(skip_serializing_if="Option::is_none")]
    /// "v2" of ED5v2
    pub version: Option<String>,
    /// Anime / Movie / ...
    pub media_type: Option<MediaType>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub language: Option<Language>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub year: Option<u32>,
}

impl SongInfo {
    pub fn credit_sentences(&self) -> Option<(String,Option<String>)> {
        let first_string = if let &Some(ref title) = &self.media_title {
            match (&self.music_type,&self.music_number) {
                (&None,_) => format!("{}",title),
                (&Some(ref m_type),&Some(ref number)) => format!("{} - {} {}",title,m_type.short(),number),
                (&Some(ref m_type),&None) => format!("{} - {}",title,m_type.long())
            }
        } else {
            return None;
        };
        let second_string = if let (&Some(ref artist),&Some(ref song_name)) = (&self.artist,&self.song_name) {
            Some(format!("{} - {}",artist,song_name))
        } else {
            None
        };
        Some((first_string,second_string))
    }
}
