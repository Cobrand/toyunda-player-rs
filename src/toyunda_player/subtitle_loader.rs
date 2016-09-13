use std::path::Path;
use ::subtitles::*;
use ::std::fs::File;
use ::std::io::{BufReader,BufRead};
use ::std::error::Error;
use ::subtitles::pos::RowPosition;

impl<'a> Load for (&'a Path,&'a Path) {
    fn into_subtitles(&self) -> Result<Subtitles, String> {
        let frm: &Path = self.0;
        let lyr: &Path = self.1;
        let mut subtitles = Subtitles::default();

        let lyr_file = try!(File::open(lyr).map_err(|e| e.description().to_string()));
        let frm_file = try!(File::open(frm).map_err(|e| e.description().to_string()));
        let lyr_file = BufReader::new(&lyr_file);
        let frm_file = BufReader::new(&frm_file);
        let mut current_sentence_options: SentenceOptions = SentenceOptions::default();
        for (line_number, lyr_line) in lyr_file.lines().enumerate() {
            let lyr_line = try!(lyr_line.map_err(|e| {
                format!("IoError when reading {} at line {} : '{}'",
                        lyr.display(),
                        line_number,
                        e.description())
            }));
            if (!lyr_line.starts_with("%") && !lyr_line.is_empty()) {
                let mut syllables: Vec<_> = lyr_line.split('&')
                    .map(|s| {
                        Syllable {
                            text: s.to_string(),
                            begin: 0,
                            end: Some(0),
                            syllable_options: None,
                        }
                    })
                    .collect::<Vec<_>>();
                if (lyr_line.starts_with("&")) {
                    syllables.remove(0);
                };
                let sentence = Sentence {
                    syllables: syllables,
                    position: RowPosition::default(),
                    sentence_options: Some(current_sentence_options),
                };
                subtitles.sentences.push(sentence);
            } else if lyr_line.starts_with("%color") {
                use utils::parse_bgr;
                let colors: Vec<_> = lyr_line.split_whitespace().collect();
                if colors.len() == 4 {
                    let alive_color = parse_bgr(colors[1]);
                    let transition_color = parse_bgr(colors[2]);
                    let dead_color = parse_bgr(colors[3]);
                    match (alive_color, transition_color, dead_color) {
                        (Ok(alive_color), Ok(transition_color), Ok(dead_color)) => {
                            if current_sentence_options.syllable_options.is_none() {
                                current_sentence_options.syllable_options =
                                    Some(SyllableOptions::default());
                            }
                            current_sentence_options.syllable_options
                                .as_mut()
                                .unwrap()
                                .alive_color = Some(alive_color);
                            current_sentence_options.syllable_options
                                .as_mut()
                                .unwrap()
                                .transition_color = Some(transition_color);
                            current_sentence_options.syllable_options
                                .as_mut()
                                .unwrap()
                                .dead_color = Some(dead_color);
                        }
                        _ => {
                            error!("Invalid %color syntax when reading {} at line {} : '{}'",
                                   lyr.display(),
                                   line_number,
                                   lyr_line);
                        }
                    }
                } else {
                    error!("Invalid %color syntax when reading {} at line {} : '{}'",
                           lyr.display(),
                           line_number,
                           lyr_line);
                }
            };
        }
        let mut frames: Vec<(u32, u32)> = vec![];
        for (line_number, frm_line) in frm_file.lines().enumerate() {
            let frm_line = try!(frm_line.map_err(|e| {
                format!("IoError when reading {} at line {} : '{}'",
                        frm.display(),
                        line_number,
                        e.description())
            }));
            if !frm_line.trim().is_empty() {
                let line_frames: Result<Vec<_>, _> = frm_line.split(' ')
                    .map(|s| s.parse::<u32>())
                    .collect();
                let begin_end = line_frames.map_err(|e| format!("{}", e))
                    .and_then(|line_frames| {
                        match (line_frames.get(0), line_frames.get(1), line_frames.get(2)) {
                            (Some(&begin), Some(&end), None) => Ok((begin, end)),
                            (None, _, _) | (_, None, _) => {
                                Err(format!("Error while parsing frm file '{}' at line {}, not \
                                             enough values",
                                            frm.display(),
                                            line_number))
                            }
                            (_, _, Some(_)) => {
                                Err(format!("Error while parsing frm file '{}' at line {}, too \
                                             many values",
                                            frm.display(),
                                            line_number))
                            }
                        }
                    });
                let begin_end = try!(begin_end);
                frames.push(begin_end);
            } else {
                warn!("empty line {} in frm file '{}'", line_number, frm.display());
            }
        }
        let mut frame_iter = frames.iter();
        'sentences: for sentence in subtitles.sentences.iter_mut() {
            for syllable in sentence.syllables.iter_mut() {
                match frame_iter.next() {
                    Some(&(begin, end)) => {
                        syllable.begin = begin;
                        syllable.end = Some(end);
                    }
                    None => {
                        break 'sentences;
                    }
                }
            }
        }
        try!(subtitles.check());
        Ok(subtitles)
    }
}
