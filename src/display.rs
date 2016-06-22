extern crate sdl2;
use std::ops::DerefMut;
use displayer::Displayer;
use font;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use std::vec::Vec;

pub trait Display {
    fn draw(self, &mut Displayer);
}

#[derive(Debug)]
pub struct TextElement {
    pub text: String,
    pub color: Color,
    pub outline: Option<Color>,
    pub shadow: Option<Color>,
}

impl TextElement {
    fn as_surface(&self, font_set: &font::FontSet) -> Surface {
        let (surface_width, surface_height) = font_set.get_outline_font()
                                                      .size_of(self.text.as_str())
                                                      .unwrap();
        let mut target_surface: Surface = Surface::new(surface_width,
                                                       surface_height,
                                                       sdl2::pixels::PixelFormatEnum::ARGB8888)
                                              .expect("Failed to create Surface with ARGB8888 \
                                                       Format");
        match self.outline {
            Some(outline_color) => {
                // blit the surface containing the border
                let outline_surface = font_set.get_outline_font()
                                              .render(self.text.as_str())
                                              .blended(outline_color)
                                              .unwrap();
                let (outline_surface_width, outline_surface_height) = outline_surface.size();
                outline_surface.blit(None,
                                     target_surface.deref_mut(),
                                     Some(Rect::new(0,
                                                    0,
                                                    outline_surface_width,
                                                    outline_surface_height)))
                               .unwrap();
            }
            None => {} // do nothing about the outline
        }
        {
            // blit the surface containing the center font
            let surface = font_set.get_regular_font()
                                  .render(self.text.as_str())
                                  .blended(self.color)
                                  .unwrap();
            let (surface_width, surface_height) = surface.size();
            surface.blit(None,
                         target_surface.deref_mut(),
                         Some(Rect::new(font::OUTLINE_WIDTH as i32, // start from OUTLINE
                                        font::OUTLINE_WIDTH as i32, // to center the text
                                        surface_width,
                                        surface_height)))
                   .unwrap();
        }

        target_surface.set_alpha_mod(match self.color {
            Color::RGB(_, _, _) => 255,
            Color::RGBA(_, _, _, a) => a,
        });
        target_surface
    }
}

#[allow(dead_code)]
#[derive(Copy,Debug,Clone)]
pub enum PosX {
    Centered,
    FromLeft(u32),
    FromRight(u32),
    FromLeftPercent(f32),
    FromRightPercent(f32),
}

#[allow(dead_code)]
#[derive(Copy,Debug,Clone)]
pub enum PosY {
    Centered,
    FromTop(u32),
    FromBottom(u32),
    FromTopPercent(f32),
    FromBottomPercent(f32),
}

#[allow(dead_code)]
#[derive(Copy,Debug,Clone)]
pub enum Size {
    // Percentage(f64),
    FitPercent(Option<f32>, Option<f32>),
    Fit(Option<u32>, Option<u32>),
}

#[derive(Debug)]
pub struct Text2D {
    pub text: Vec<TextElement>,
    pub size: Size,
    pub pos: (PosX, PosY),
    pub anchor: (f32, f32),
}

impl Text2D {
    pub fn to_string(&self) -> String {
        self.text.iter().fold(String::default(),
                              |accu_text, text_element| accu_text + &text_element.text)
    }
}


impl Display for Text2D {
    fn draw(self, displayer: &mut Displayer) {
        let (window_width, window_height) = displayer.sdl_renderer().window().unwrap().size();
        let (fit_width, fit_height) = match self.size {
            Size::FitPercent(option_x, option_y) => displayer.sub_screen_dims(option_x, option_y),
            Size::Fit(x, y) => (x, y),
        };

        let target_texture = {
            let is_outline_enabled = self.text
                                         .iter()
                                         .any(|text_element| text_element.outline.is_some());
            let all_text = self.to_string();
            let font_set = displayer.fonts()
                                    .get_fittest_font_set(all_text.as_str(),
                                                          (fit_width, fit_height),
                                                          is_outline_enabled)
                                    .unwrap();
            let (target_surface_width, target_surface_height) = font_set.get_outline_font()
                                                                        .size_of(all_text.as_str())
                                                                        .expect("Unable to get \
                                                                                 outline pixel \
                                                                                 size of str");
            // ARGB8888 because it's the only one supported on my computer; i hope it's the same everywhere else ?
            let mut target_surface = Surface::new(target_surface_width,
                                                  target_surface_height,
                                                  sdl2::pixels::PixelFormatEnum::ARGB8888)
                                         .expect("Failed to create Surface with ARGB8888 Format");
            let mut width_offset: i32 = if is_outline_enabled {
                0
            } else {
                font::OUTLINE_WIDTH as i32
            };
            for text_element in self.text.iter() {
                // for each text element, blit it over
                let text_surface = text_element.as_surface(font_set);
                let (text_surface_w, text_surface_h) = text_surface.size();
                text_surface.blit(None,
                                  target_surface.deref_mut(),
                                  Some(Rect::new(width_offset, 0, text_surface_w, text_surface_h)))
                            .unwrap();
                width_offset = width_offset + text_surface_w as i32 -
                               (font::OUTLINE_WIDTH as i32 * 2);
            }
            let target_texture = displayer.sdl_renderer()
                                          .create_texture_from_surface(target_surface)
                                          .expect("Unable to create empty texture with pixel \
                                                   format ARGB8888");
            target_texture
        };
        let TextureQuery { width:texture_width, height:texture_height, .. } =
            target_texture.query();
        // POSITION
        let (pos_x, pos_y) = self.pos;
        let mut target_rect: Rect = Rect::new(0, 0, texture_width, texture_height);
        let delta_anchor_x = (self.anchor.0 * texture_width as f32) as i32;
        let delta_anchor_y = (self.anchor.1 * texture_height as f32) as i32;
        match pos_x {
            PosX::Centered => target_rect.set_x((window_width / 2) as i32 - delta_anchor_x),
            PosX::FromLeft(value) => target_rect.set_x(value as i32 - delta_anchor_x),
            PosX::FromLeftPercent(percent) => {
                target_rect.set_x((percent * (window_width as f32)) as i32 - delta_anchor_x)
            }
            PosX::FromRight(value) => {
                target_rect.set_x(window_width as i32 - value as i32 - delta_anchor_x)
            }
            PosX::FromRightPercent(percent) => {
                target_rect.set_x(window_width as i32 - (percent * (window_width as f32)) as i32 -
                                  delta_anchor_x)
            }
        };
        match pos_y {
            PosY::Centered => target_rect.set_y((window_height / 2) as i32 - delta_anchor_y),
            PosY::FromTop(value) => target_rect.set_y(value as i32 - delta_anchor_y),
            PosY::FromTopPercent(percent) => {
                target_rect.set_y((percent * (window_height as f32)) as i32 - delta_anchor_y)
            }
            PosY::FromBottom(value) => {
                target_rect.set_y(window_height as i32 - value as i32 - delta_anchor_y)
            }
            PosY::FromBottomPercent(percent) => {
                target_rect.set_y(window_height as i32 - (percent * (window_height as f32)) as i32 -
                                  delta_anchor_y)
            }
        };
        displayer.sdl_renderer_mut().copy(&target_texture, None, Some(target_rect));
    }
}
