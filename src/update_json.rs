use toyunda_player::VideoMeta;
use subtitles::*;

use clap::ArgMatches;
use std::path::PathBuf;
use std::fs::File;

extern crate serde_json;
extern crate serde_yaml;

/// true on success
/// false on failure
pub fn update_json(args: &ArgMatches) -> bool {
    if let Some(path) = args.value_of("JSON_FILE") {
        let json_path = PathBuf::from(path);
        let yaml_path = json_path.with_extension("yaml");
        if (json_path.is_file() && yaml_path.is_file()) {
            let json_file = File::open(&json_path);
            let yaml_file = File::open(&yaml_path);
            match (json_file, yaml_file) {
                (Ok(json_file), Ok(yaml_file)) => {
                    let video_meta: Result<VideoMeta, _> = serde_yaml::from_reader(&yaml_file);
                    let subs: Result<Subtitles, _> = serde_json::from_reader(&json_file);
                    match (video_meta, subs) {
                        (Ok(video_meta), Ok(mut subs)) => {
                            subs.song_info = video_meta.song_info.clone();
                            let mut json_file = File::create(&json_path)
                                .expect("Can't open json file for writing");
                            if let Err(e) = serde_json::to_writer_pretty(&mut json_file, &subs) {
                                println!("Some error occured while generating new subtitles : \
                                          {:?}",
                                         e);
                                false
                            } else {
                                true
                            }
                        }
                        (Err(err), _) => {
                            println!("error while parsing video_meta : {:?}", err);
                            false
                        }
                        (_, Err(err)) => {
                            println!("error while parsing subtitles : {:?}", err);
                            false
                        }
                    }
                }
                (Err(e), _) => {
                    println!("file `{}` couldn't be opened : {:?}",
                             json_path.display(),
                             e);
                    false
                }
                (_, Err(e)) => {
                    println!("file `{}` couldn't be opened : {:?}",
                             yaml_path.display(),
                             e);
                    false
                }
            }
        } else {
            if json_path.is_file() {
                println!("file `{}` not found", yaml_path.display());
            } else {
                println!("file `{}` not found", json_path.display());
            };
            false
        }
    } else {
        println!("A file is required for the subcommand 'update'");
        // clap shouldn't let this case happen but never too sure
        false
    }
}
