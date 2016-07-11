use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::vec::Vec;

pub struct Subtitles {
    subtitles: Vec<Vec<(String, (u32, u32))>>,
}

impl Subtitles {
    pub fn new(subtitles: Vec<Vec<(String, (u32, u32))>>) -> Self {
        Subtitles { subtitles: subtitles }
    }
    pub fn get_subtitles(&self) -> &Vec<Vec<(String, (u32, u32))>> {
        &self.subtitles
    }
    pub fn get_subtitles_mut(&mut self) -> &Vec<Vec<(String, (u32, u32))>> {
        &mut self.subtitles
    }
}

fn load_frm(frm_path: &Path) -> Vec<(u32, u32)> {
    let frm_file = File::open(frm_path).unwrap();
    let mut frm_lines = Vec::new();
    let frm_buf_reader = BufReader::new(&frm_file);
    for (i, frm_line) in frm_buf_reader.lines().enumerate() {
        let frm_line = frm_line.unwrap();
        if (!frm_line.is_empty()) {
            let frm_line_frames = frm_line.split(' ')
                                          .map(|s| {
                                              s.parse::<u32>()
                                               .expect(format!("failed to parse frm file at line \
                                                                {}",
                                                               i)
                                                           .as_str())
                                          })
                                          .collect::<Vec<u32>>();
            let (frm_begin, frm_end) = (frm_line_frames[0], frm_line_frames[1]);
            if (frm_begin > frm_end) {
                error!("error while parsing frm file, end frame is sooner than begin frame");
            }
            frm_lines.push((frm_begin, frm_end));
        }
    }
    frm_lines
}

fn load_lyr(lyr_path: &Path) -> Vec<Vec<String>> {
    let lyr_file = File::open(lyr_path).unwrap();
    let lyr_buf_reader = BufReader::new(&lyr_file);
    let mut lyr_lines: Vec<Vec<String>> = Vec::new();
    for lyr_line in lyr_buf_reader.lines() {
        let lyr_line = lyr_line.expect("Error unwraping line");
        if (!lyr_line.starts_with("%") && !lyr_line.is_empty()) {
            // println!("{:?}",line.split('&').map(|s|{s.to_string()}).collect::<Vec<_>>());
            let mut lyr_line_vec: Vec<String> = lyr_line.split('&')
                                                        .map(|s| s.to_string())
                                                        .collect::<Vec<_>>();
            if (lyr_line.starts_with("&")) {
                lyr_line_vec.remove(0);
            }
            lyr_lines.push(lyr_line_vec);
        }
    }
    lyr_lines
}

pub fn load_subtitles(lyr_path: &Path, frm_path: &Path) -> Vec<Vec<(String, (u32, u32))>> {
    let frames = load_frm(frm_path);
    let lyr_lines = load_lyr(lyr_path);
    let mut lines: Vec<Vec<(String, (u32, u32))>> = Vec::new();
    let mut current_syllabus = 0;
    for lyr_line in lyr_lines.into_iter() {
        let syllabus_left = frames.split_at(current_syllabus).1;
        current_syllabus += lyr_line.len();
        let zipped_syllabus = lyr_line.into_iter()
                                      .zip(syllabus_left.iter().cloned())
                                      .collect::<Vec<(String, (u32, u32))>>();
        lines.push(zipped_syllabus);

    }
    lines
}
