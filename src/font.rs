extern crate sdl2_ttf;

use std::ops::Index;
use std::cmp::Ordering;
use std::path::Path;

pub const OUTLINE_WIDTH : u16 = 2 ;

pub struct FontSet {
    /// size of the loaded font
    font_size: u16,
    /// Font object without outline
    font_regular: sdl2_ttf::Font,
    /// Font object with outline
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
        let mut font_size = 3;
        let font_size_max = 128;
        let font_size_increment = 1;
        let mut error: bool = false;
        'fontlist: while (font_size < font_size_max) {
            let mut font_bold;
            let font_regular = ttf_context.load_font(font_path, font_size)
                .expect("Unable to load font");
            match ttf_context.load_font(font_path, font_size) {
                Ok(font_set) => {
                    font_bold = font_set;
                }
                Err(_) => {
                    error = true;
                    break 'fontlist;
                }
            }
            font_bold.set_outline_width(OUTLINE_WIDTH);
            result.fonts.push(FontSet {
                font_size: font_size,
                font_regular: font_regular,
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

    /// Given a string and a maximum width, get the fittest font from the FontList
    pub fn get_fittest_font_set(&self, string:&str,max_dims:(Option<u32>,Option<u32>),outline:bool) -> Result<&FontSet, ()> {
        if max_dims == (None,None) {
            Err(()) // cant get the fittiest if both are None !
        } else {
            match self.fonts.len() {
                0 => Err(()),
                1 => Ok(self.fonts.first().unwrap()),
                _ => {
                    let search_result = self.fonts.binary_search_by(|fontset| {
                        let ref font_bold = fontset.font_bold;
                        let ref font_regular = fontset.font_regular;
                        let string_dims =
                            if outline {font_bold} else {font_regular}
                            .size_of(string)
                            .expect("Failed to get size of some string");
                        match max_dims {
                            (Some(width),Some(height)) => {
                                match (string_dims.0.cmp(&width),string_dims.1.cmp(&height)) {
                                    (Ordering::Greater,_) | (_,Ordering::Greater) => Ordering::Greater,
                                    (Ordering::Equal,_) | (_,Ordering::Equal) => Ordering::Equal,
                                    _ => Ordering::Less
                                }
                            },
                            (Some(width),None) => {
                                string_dims.0.cmp(&width)
                            },
                            (None,Some(height)) => {
                                string_dims.1.cmp(&height)
                            },
                            (None,None) => unreachable!()
                        }
                    });
                    match search_result {
                        Ok(index) => Ok(&self.fonts[index]),
                        Err(0) => Ok(&self.fonts[0]),
                        Err(index) if index == self.fonts.len() => Ok(&self.fonts.last().unwrap()),
                        Err(index) => Ok(&self.fonts[index - 1])
                        // it should fit, meaning that if we can't find something exactly we should
                        // take the first that fits
                    }
                }
            }
        }
    }
}
