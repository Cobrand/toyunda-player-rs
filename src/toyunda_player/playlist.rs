use ::subtitles::metainfo::MetaInfo;
use std::collections::VecDeque;
use std::path::{Path,PathBuf};

pub type Playlist = VecDeque<VideoMeta> ;

#[derive(Debug,Serialize)]
pub struct VideoMeta {
    pub video_path : PathBuf,
    pub meta_info : MetaInfo
}

impl VideoMeta {
    pub fn new(path:PathBuf) -> VideoMeta {
        VideoMeta {
            video_path : path,
            meta_info : MetaInfo::default()
        }
        // TODO read meta_info from yaml
    }

    pub fn from_path<P : AsRef<Path>>(path:P) -> VideoMeta {
        VideoMeta {
            video_path : PathBuf::from(path.as_ref()),
            meta_info : MetaInfo::default()
        }
    }
}
