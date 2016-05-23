extern crate sdl2_ttf;

use std::ops::Index;
use std::cmp::Ordering;
use std::path::Path;

pub struct FontSet {
    font_size: u16,
    font_regular: sdl2_ttf::Font,
    font_bold: sdl2_ttf::Font,
}

impl Eq for FontSet {}

impl PartialEq for FontSet {
    fn eq(&self, other: &Self) -> bool {
        self.font_size.eq(&other.font_size)
    }
}

impl PartialOrd for FontSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.font_size.partial_cmp(&other.font_size)
    }
}

impl Ord for FontSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.font_size.cmp(&other.font_size)
    }
}

impl Index<usize> for FontList {
    type Output = FontSet;
    fn index(&self, index: usize) -> &FontSet {
        &self.get_font_set(index).unwrap()
    }
}

impl FontSet {
    pub fn get_size(&self) -> u16 {
        self.font_size
    }
    pub fn get_regular_font(&self) -> &sdl2_ttf::Font {
        &self.font_regular
    }
    pub fn get_outline_font(&self) -> &sdl2_ttf::Font {
        &self.font_bold
    }
}

// MADE PUBLIC FOR TESTS, MAKE PRIVATE WHEN NOT NECESSARY ANYMORE
pub struct FontList {
    // font list is a SORTED font list
    fonts: Vec<FontSet>,
}

impl FontList {
    pub fn new(font_path: &Path, ttf_context: &sdl2_ttf::Sdl2TtfContext) -> Result<FontList, ()> {
        let mut result = FontList { fonts: Vec::<FontSet>::new() };
        let mut font_size = 4;
        let font_size_max = 192;
        let font_size_increment = 2;
        let mut error: bool = false;
        'fontlist: while (font_size < font_size_max) {
            let mut font_bold;
            match ttf_context.load_font(font_path, font_size) {
                Ok(font_set) => {
                    font_bold = font_set;
                }
                Err(_) => {
                    error = true;
                    break 'fontlist;
                }
            }
            font_bold.set_outline_width(2);
            result.fonts.push(FontSet {
                font_size: font_size,
                font_regular: ttf_context.load_font(font_path, font_size).unwrap(),
                font_bold: font_bold,
            });
            font_size += font_size_increment;
        }
        if error {
            Err(())
        } else {
            Ok(result)
        }
    }

    pub fn get_font_set(&self, index: usize) -> Option<&FontSet> {
        self.fonts.get(index)
    }

    pub fn add_font_set(&mut self, font_set: FontSet) -> Result<(), &FontSet> {
        let result = self.fonts
                         .binary_search_by(|fontset| fontset.font_size.cmp(&font_set.font_size));
        match result {
            Ok(index) => Err(self.fonts.get(index).unwrap()),
            Err(index) => {
                self.fonts.insert(index, font_set);
                Ok(())
            }
        }
    }

    pub fn get_closest_font_set(&self, font_size: u16) -> Result<&FontSet, ()> {
        match self.fonts.len() {
            0 => Err(()),
            1 => Ok(self.fonts.first().unwrap()),
            _ => {
                let search_result = self.fonts.binary_search_by(|fontset| {
                    fontset.font_size.cmp(&font_size)
                });
                match search_result {
                    Ok(index) => Ok(&self.fonts[index]),
                    Err(0) => Ok(&self.fonts[0]),
                    Err(index) if index == self.fonts.len() => Ok(&self.fonts.last().unwrap()),
                    Err(index) => {
                        let font_set_min = &self.fonts[index - 1];
                        let font_set_max = &self.fonts[index];
                        if (font_set_max.font_size - font_size >
                            font_size - font_set_min.font_size) {
                            Ok(font_set_min)
                        } else {
                            Ok(font_set_max)
                        }
                    }
                }
            }
        }
    }
}
