extern crate sdl2;
use ::subtitles::Color;
use ::sdl2::pixels::Color as SdlColor;
use std::cmp::{max, min};

use std::path::{Path, PathBuf};
use std::{io, fs};

/// will always return a flat color regardless or alpha
pub fn mix_colors(color1: SdlColor, color2: SdlColor, mix_ratio: f32) -> SdlColor {
    let (r1, g1, b1) = color1.rgb();
    let (r2, g2, b2) = color2.rgb();
    let (r1, g1, b1) = (r1 as i16, g1 as i16, b1 as i16);
    let (r2, g2, b2) = (r2 as i16, g2 as i16, b2 as i16);
    let (r, g, b): (i16, i16, i16) = (((r2 - r1) as f32 * mix_ratio) as i16 + r1,
                                      ((g2 - g1) as f32 * mix_ratio) as i16 + g1,
                                      ((b2 - b1) as f32 * mix_ratio) as i16 + b1);

    let (r, g, b) =
        (max(min(r, 0xFF), 0) as u8, max(min(g, 0xFF), 0) as u8, max(min(b, 0xFF), 0) as u8);
    SdlColor::RGB(r, g, b)
}

pub fn fade_color(color1: SdlColor, fade_ratio: f32) -> SdlColor {
    let (r, g, b, a): (u8, u8, u8, u8) = match color1 {
        SdlColor::RGB(r, g, b) => (r, g, b, 255),
        SdlColor::RGBA(r, g, b, a) => (r, g, b, a),
    };
    SdlColor::RGBA(r, g, b, (a as f32 * fade_ratio) as u8)
}

pub fn parse_hex(hex: &str) -> Result<u32, String> {
    let mut parsed: u32 = 0;
    for character in hex.chars() {
        let char_value: u32 = try!(character.to_digit(16)
            .ok_or(String::from("failed to parse hexadecimal character")));
        parsed = parsed * 0x10 + char_value;
    }
    Ok(parsed)
}

pub fn parse_bgr(bgr: &str) -> Result<Color, String> {
    let bgr = bgr.trim();
    if bgr.len() != 6 {
        Err(String::from("Invalid BGR format"))
    } else {
        let (blue, greenred) = bgr.split_at(2);
        let (green, red) = greenred.split_at(2);
        let blue = try!(parse_hex(blue)) as u8;
        let green = try!(parse_hex(green)) as u8;
        let red = try!(parse_hex(red)) as u8;
        Ok(Color {
            red: red,
            green: green,
            blue: blue,
        })
    }
}

pub fn for_each_in_dir<P: AsRef<Path>, F: Fn(&Path) -> bool>(directory: P,
                                                             recursion_level: u32,
                                                             filter: &F)
                                                             -> (Vec<PathBuf>, Vec<io::Error>) {
    if (recursion_level == 0) {
        return (vec![], vec![]);
    }
    let mut vec_path: Vec<PathBuf> = Vec::new();
    let mut vec_error: Vec<io::Error> = Vec::new();
    let paths = match fs::read_dir(directory) {
        Ok(paths) => paths,
        Err(e) => {
            return (vec![], vec![e]);
            unreachable!()
        }
    };
    for path in paths {
        match path {
            Ok(path) => {
                let filetype = path.file_type();
                match filetype {
                    Ok(finfo) => {
                        let pathbuf = path.path();
                        if finfo.is_dir() {
                            let (v_p, v_e) = for_each_in_dir(&pathbuf, recursion_level - 1, filter);
                            vec_path.extend(v_p);
                            vec_error.extend(v_e);
                        } else if filter(&pathbuf) {
                            vec_path.push(pathbuf);
                        }
                    }
                    Err(e) => {
                        vec_error.push(e);
                    }
                }
            }
            Err(e) => {
                vec_error.push(e);
            }
        }
    }
    (vec_path, vec_error)
}

#[test]
fn test_bgr() {
    let sample_bgr = "FF0000";
    assert_eq!(parse_bgr(sample_bgr).unwrap(), Color::RGB(0, 0, 255));
}

#[test]
fn test_parse_hex() {
    let sample_hex = "201";
    assert_eq!(parse_hex(sample_hex).unwrap(), 513);
}
