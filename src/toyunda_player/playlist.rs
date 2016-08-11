use std::sync::{RwLock,Arc};
use ::subtitles::metainfo::MetaInfo;
use std::collections::VecDeque;
use std::path::{Path,PathBuf};

use ::toyunda_player::error::*;

pub type Playlist = FifoSync<VideoMeta> ;

pub struct FifoSync<T> {
    vec:Arc<RwLock<VecDeque<T>>>
}

impl<T> FifoSync<T> {
    pub fn new() -> FifoSync<T> {
        FifoSync {
            vec:Arc::new(RwLock::new(VecDeque::new()))
        }
    }

    pub fn push_back(&self,value : T) -> Result<()> {
        try!(self.vec.write().map_err(|_| Error::Text(String::from("Guard Poisoning"))))
            .push_back(value);
        Ok(())
    }

    pub fn pop_front(&self) -> Result<Option<T>> {
        let value = try!(self.vec.write().map_err(|_| Error::Text(String::from("Guard Poisoning"))))
            .pop_front();
        Ok(value)
    }

    pub fn clear(&self) -> Result<()> {
        try!(self.vec.write().map_err(|_| Error::Text(String::from("Guard Poisoning"))))
            .clear();
        Ok(())
    }

    pub fn for_each<F : Fn(&T)>(&self,fun: F) -> Result<()> {
        let vec = try!(self.vec.read().map_err(|_| Error::Text(String::from("Guard Poisoning"))));
        for video_meta in vec.iter() {
            fun(video_meta);
        }
        Ok(())
    }
}

#[derive(Debug)]
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
