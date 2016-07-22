use std::ops::DerefMut;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use sdl2::rect::Rect;

#[derive(Debug)]
pub struct TextElement {
    pub text: String, // TODO : replace with &str
    pub color: Color,
    pub outline: Option<Color>,
    pub shadow: Option<Color>,
}

impl TextElement {
    pub fn as_surface(&self, font_set: &::display::font::FontSet) -> Surface {
        let (surface_width, surface_height) = font_set.get_outline_font()
                                                      .size_of(self.text.as_str())
                                                      .unwrap();
        let mut target_surface: Surface = Surface::new(surface_width,
                                                       surface_height,
                                                       ::sdl2::pixels::PixelFormatEnum::ARGB8888)
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
                         Some(Rect::new(::display::font::OUTLINE_WIDTH as i32, // start from OUTLINE
                                        ::display::font::OUTLINE_WIDTH as i32, // to center the text
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
