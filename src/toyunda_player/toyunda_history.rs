// this file is used for serializing / deserializing 

extern crate serde_json;

use std::collections::HashMap;
use std::ops::{Deref,DerefMut};
use chrono::{DateTime,Local,FixedOffset};
use std::path::Path;
use std::fs::{File,OpenOptions};

pub struct SongsHistory {
    _file:File,
    hashmap:HashMap<String,Vec<DateTime<Local>>>
}

impl SongsHistory {
    pub fn new<P:AsRef<Path>>(path:P) -> Result<SongsHistory,String> {
        let file =
            OpenOptions::new()
                        .write(true)
                        .create(true)
                        .read(true)
                        .open(&path);
        let file = try!(file.map_err(|e| format!("Fileopen error : {}",e)));
        let file_len : u64 = try!(file.metadata().map_err(|e| format!("File metadata read error : {}",e))).len();
        let json_hashmap : Result<HashMap<String,Vec<String>>,_> = if file_len < 2 {
            serde_json::from_str("{}")
        } else {
            serde_json::from_reader(&file)
        };
        let json_hashmap = try!(json_hashmap.map_err(|s|format!("Hashmap parse error : {}",s)));
        let hashmap : HashMap<String,Vec<DateTime<Local>>> =
            json_hashmap
                .into_iter()
                .map(|(key, values)| {
                    let newvalues : Vec<DateTime<Local>> = values.iter().filter_map(|string|{
                        match DateTime::<FixedOffset>::parse_from_rfc3339(&*string).map(|d| d.with_timezone(&Local)) {
                            Ok(d) => Some(d),
                            Err(e) => {
                                error!("Error when parsing date `{}` from {} : {}", string, path.as_ref().display(), e);
                                None
                            }
                        }
                    }).collect();
                    (key, newvalues)
                })
                .collect();
        Ok(SongsHistory {
            _file:file,
            hashmap: hashmap
        })
    }

    pub fn insert_song_history_entry(&mut self,name:&str) {
        if !self.contains_key(name) {
            self.insert(String::from(name),Vec::with_capacity(1));
        };
        if let Some(ref mut vec) = self.get_mut(name) {
            vec.push(Local::now())
        } else {
            unreachable!()
        };
    }

    pub fn save(&mut self) {
        use std::io::{Seek,SeekFrom};
        // TODO error handling, at least print the IoErrors !
        let _ = self._file.seek(SeekFrom::Start(0));
        let _ = self._file.set_len(0);
        let json_hashmap : HashMap<String,Vec<String>> =
            self.hashmap
                .iter()
                .map(|(key, values)| {
                let newvalues : Vec<String> = values.into_iter().map(|d| d.to_rfc3339()).collect();
                (key.clone(), newvalues)
            })  .collect();
        if let Err(e) = serde_json::to_writer_pretty(&mut self._file,&json_hashmap) {
            error!("Failed to write songs_history file. You probably lost everything. Sorry.\n{}",e);
        };
    }
}

impl Drop for SongsHistory {
    fn drop(&mut self) {
        self.save()
    }
}

impl Deref for SongsHistory {
    type Target = HashMap<String,Vec<DateTime<Local>>>;
    fn deref(&self) -> &HashMap<String,Vec<DateTime<Local>>> {
        &self.hashmap
    }
}

impl DerefMut for SongsHistory {
    fn deref_mut(&mut self) -> &mut HashMap<String,Vec<DateTime<Local>>> {
        &mut self.hashmap
    }
}
