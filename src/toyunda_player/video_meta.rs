extern crate serde_yaml;

use ::subtitles::metainfo::MetaInfo;
use std::path::{Path, PathBuf};

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct VideoMeta {
    pub video_path: PathBuf,
    pub json_path: Option<PathBuf>,
    pub lyr_path: Option<PathBuf>,
    pub frm_path: Option<PathBuf>,
    #[serde(default)] 
    pub meta_info: MetaInfo,
}

impl VideoMeta {
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<VideoMeta, String> {
        let path = path.as_ref();
        match ::std::fs::File::open(path) {
            Ok(yaml_file) => {
                serde_yaml::from_reader(yaml_file)
                    .map_err(|err| format!("Error when parsing .yaml file : {}", err))
            }
            Err(e) => {
                error!("Error when loading yaml file '{}' : {}", path.display(), e);
                Err(format!("Error when loading yaml file '{}'; file could not be opened",
                            path.display()))
            }
        }
    }

    /// both .yaml or .mp4/.avi/... should work
    pub fn new<P: AsRef<Path>>(path: P) -> Result<VideoMeta, String> {
        let path = path.as_ref();
        match path.extension() {
            None => {
                Err(format!("Error when trying to load file '{}', file has no extension",
                            path.display()))
            }
            Some(s) if s == "yaml" => VideoMeta::from_yaml(path).map(|v_m| v_m.fix_paths(path)),
            Some(s) if s == "avi" || s == "mp4" => {
                let yaml_file: PathBuf = path.with_extension("yaml");
                if yaml_file.exists() {
                    VideoMeta::from_yaml(yaml_file).map(|v_m| v_m.fix_paths(path))
                } else {
                    Ok(VideoMeta {
                        video_path: PathBuf::from(path),
                        lyr_path: None,
                        frm_path: None,
                        json_path: None,
                        meta_info: MetaInfo::default(),
                    })
                }
            }
            Some(ext) => {
                Err(format!("Unrecognized extension '{}'",
                            ext.to_str().unwrap_or("[NON-UTF-8 SEQ]")))
            }
        }
    }

    // maps the paths in video meta so they have paths
    // absolute or relative to cwd
    pub fn fix_paths<P: AsRef<Path>>(mut self, original: P) -> VideoMeta {
        // applying join with an absolute directory overrides the origina one
        // which applies perfectly to our use case
        fn fix_path<P: AsRef<Path>>(original: P, target: &mut PathBuf) {
            // TODO remove .unwrap (so it doesnt crash for a single wrong file)
            *target = original.as_ref().parent().unwrap().join(&target);
        }

        fn fix_option_path<P: AsRef<Path>>(original: P, target: &mut Option<PathBuf>) {
            match *target {
                None => {}
                Some(ref mut target) => {
                    fix_path(original, target);
                }
            }
        }
        fix_path(&original, &mut self.video_path);
        fix_option_path(&original, &mut self.json_path);
        fix_option_path(&original, &mut self.frm_path);
        fix_option_path(&original, &mut self.lyr_path);
        self
    }

    // TODO return a &Path or &'a Path or whatever
    // same for the 2 under
    pub fn json_path(&self) -> PathBuf {
        match self.json_path {
            None => self.video_path.with_extension("json"),
            Some(ref path) => path.clone(),
        }
    }

    pub fn lyr_path(&self) -> PathBuf {
        match self.lyr_path {
            None => self.video_path.with_extension("lyr"),
            Some(ref path) => path.clone(),
        }
    }

    pub fn frm_path(&self) -> PathBuf {
        match self.frm_path {
            None => self.video_path.with_extension("frm"),
            Some(ref path) => path.clone(),
        }
    }
}
