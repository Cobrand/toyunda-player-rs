extern crate sdl2_ttf;
use std::ops::Index;
use std::cmp::Ordering;

pub struct FontSet {
    font_size: u16,
    /// Font object without outline
    font_regular: sdl2_ttf::Font<'static>,
    /// Font object with light outline
    font_lightbold: sdl2_ttf::Font<'static>,
    /// Font object with outline
    font_bold: sdl2_ttf::Font<'static>,
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
    #[inline]
    pub fn get_regular_font(&self) -> &sdl2_ttf::Font {
        &self.font_regular
    }

    #[inline]
    pub fn get_outline_font(&self) -> &sdl2_ttf::Font {
        &self.font_bold
    }

    #[inline]
    pub fn get_light_outline_font(&self) -> &sdl2_ttf::Font {
        &self.font_lightbold
    }

    pub fn get_outline_width(font_size:u16,width:usize) -> u16 {
        match width {
            0 => 0,
            1 => (font_size / 30) + 1,
            2 => (font_size / 20) + 2,
            _ => 0
        }
    }

    pub fn get_font_size(&self) -> u16 {
        self.font_size
    }
}

// MADE PUBLIC FOR TESTS, MAKE PRIVATE WHEN NOT NECESSARY ANYMORE
pub struct FontList {
    // font list is a SORTED font list
    fonts: Vec<FontSet>,
}

const DEJAVUSANS_MONO_BYTES: &'static [u8] = include_bytes!("../../res/DejaVuSansMono-Bold-WithJap.ttf");

impl FontList {
    pub fn new(ttf_context: &sdl2_ttf::Sdl2TtfContext) -> Result<FontList, String> {
        use sdl2::rwops::RWops;
        let mut result = FontList { fonts: Vec::<FontSet>::new() };
        let mut font_size = 3;
        let font_size_max = 128;
        let font_size_increment = 1;
        'fontlist: while (font_size < font_size_max) {
            let rwops_regular : RWops<'static> = try!(RWops::from_bytes(DEJAVUSANS_MONO_BYTES));
            let rwops_bold : RWops<'static> = try!(RWops::from_bytes(DEJAVUSANS_MONO_BYTES));
            let rwops_lbold : RWops<'static> = try!(RWops::from_bytes(DEJAVUSANS_MONO_BYTES));
            let font_regular =
                try!(ttf_context.load_font_from_rwops(rwops_regular, font_size));
            let mut font_bold = try!(ttf_context.load_font_from_rwops(rwops_bold, font_size));
            let mut font_lbold = try!(ttf_context.load_font_from_rwops(rwops_lbold, font_size));
            font_lbold.set_outline_width(FontSet::get_outline_width(font_size,1));
            font_bold.set_outline_width(FontSet::get_outline_width(font_size,2));
            result.fonts.push(FontSet {
                font_size: font_size,
                font_regular: font_regular,
                font_bold: font_bold,
                font_lightbold: font_lbold,
            });
            font_size += font_size_increment;
        }
        Ok(result)
    }

    pub fn get_font_set(&self, index: usize) -> Option<&FontSet> {
        self.fonts.get(index)
    }

    /// Given a string and a maximum width, get the fittest font from the FontList
    pub fn get_fittest_font_set_id(&self,
                                   string: &str,
                                   max_dims: (Option<u32>, Option<u32>),
                                   outline: usize)
                                   -> Result<usize, String> {
        if max_dims == (None, None) {
            Err(String::from("can't get fittiest font if both dims are None")) // cant get the fittiest if both are None !
        } else {
            match self.fonts.len() {
                0 => Err(String::from("can't get the fittest font if there is none available")),
                1 => Ok(0),
                _ => {
                    let search_result = self.fonts.binary_search_by(|fontset| {
                        let search_font = match outline {
                            0 => &fontset.font_regular,
                            1 => &fontset.font_lightbold,
                            2 => &fontset.font_bold,
                            _ => {warn!("wrong font_set size set");&fontset.font_regular},
                        };
                        let string_dims = search_font.size_of(string)
                            .expect("Failed to get dimensions");
                        match max_dims {
                            (Some(width), Some(height)) => {
                                match (string_dims.0.cmp(&width), string_dims.1.cmp(&height)) {
                                    (Ordering::Greater, _) |
                                    (_, Ordering::Greater) => Ordering::Greater,
                                    (Ordering::Equal, _) |
                                    (_, Ordering::Equal) => Ordering::Equal,
                                    _ => Ordering::Less,
                                }
                            }
                            (Some(width), None) => string_dims.0.cmp(&width),
                            (None, Some(height)) => string_dims.1.cmp(&height),
                            (None, None) => unreachable!(),
                        }
                    });
                    match search_result {
                        Ok(index) => Ok(index),
                        Err(0) => Ok(0),
                        Err(index) if index == self.fonts.len() => Ok(self.fonts.len() - 1),
                        Err(index) => Ok(index - 1),
                        // it should fit, meaning that if we can't find something exactly we should
                        // take the first that fits
                    }
                }
            }
        }
    }
}
