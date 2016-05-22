extern crate sdl2;
extern crate sdl2_ttf;
use sdl2::render::{Renderer, TextureQuery, BlendMode};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::vec::Vec;
use std::cmp::Ordering;
use std::path::Path;
use std::ops::Index;

use std::ops::DerefMut;

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
struct FontList {
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

pub struct Displayer<'a> {
    fonts: FontList,
    renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: sdl2_ttf::Sdl2TtfContext,
}

impl<'a> Displayer<'a> {
    pub fn new(mut renderer: Renderer<'a>) -> Result<Displayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = sdl2_ttf::init().unwrap();
        let font_list = FontList::new(Path::new("./res/DejaVuSansMono-Bold.ttf"),
                                      &ttf_context)
                            .unwrap();
        let displayer = Displayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
        };
        Ok(displayer)
    }

    pub fn display(&mut self, text: &str) {
        let size: f32 = 0.039;
        let window_width = self.renderer.window().unwrap().size().0 as f32;
        let font_set = self.fonts.get_closest_font_set((size * window_width) as u16).unwrap();
        let font = font_set.get_regular_font();
        let font_outline = font_set.get_outline_font();
        let surface = font.render(text)
                          .blended(Color::RGB(180, 180, 180))
                          .unwrap();
        let mut surface_outline = font_outline.render(text)
                                              .blended(Color::RGB(0, 0, 0))
                                              .unwrap();
        let outline_width: u32 = 2;
        let (width, height) = surface_outline.size();

        surface.blit(None,
                     surface_outline.deref_mut(),
                     Some(Rect::new(outline_width as i32,
                                    outline_width as i32,
                                    (width - outline_width),
                                    (height - outline_width)))).expect("Failed to blit texture");
        let mut texture = self.renderer.create_texture_from_surface(&surface_outline).unwrap();
        texture.set_blend_mode(BlendMode::Blend);
        texture.set_alpha_mod(128);
        let TextureQuery { width:texture_width, height:texture_height, .. } = texture.query();
        self.renderer.copy(&mut texture,
                           None,
                           Some(Rect::new(3, 3, texture_width, texture_height)));
    }

    pub fn render(&mut self) {
        self.sdl_renderer_mut().window().unwrap().gl_swap_window();
    }

    pub fn sdl_renderer_mut(&mut self) -> &mut Renderer<'a> {
        &mut self.renderer
    }

    pub fn sdl_renderer(&self) -> &Renderer<'a> {
        &self.renderer
    }
}
