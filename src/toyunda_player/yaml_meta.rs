use ::toyunda_player::video_meta::* ;
use std::path::PathBuf;

#[derive(Debug,Serialize,Clone)]
pub struct YamlMeta {
    pub yaml_path: PathBuf,
    pub video_meta: VideoMeta,
}
