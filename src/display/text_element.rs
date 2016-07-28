use std::ops::DerefMut;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use sdl2::render::{Texture,TextureQuery,Renderer,BlendMode};
use sdl2::rect::Rect;

#[derive(Debug)]
pub struct TextElement {
    pub text: String, // TODO : replace with &str
    pub color: Color,
    pub outline: Option<Color>,
    pub shadow: Option<Color>,
    pub attach_logo: bool
}

impl TextElement {
    pub fn blit(&self,
                font_set: &::display::font::FontSet,
                renderer: &mut Renderer,
                origin: (i32,i32)) -> Rect {
        use ::sdl2::pixels::PixelFormatEnum::ARGB8888;
        let outline_width = ::display::font::OUTLINE_WIDTH as u32;
        let regular_surface = font_set.get_regular_font()
                                      .render(self.text.as_str())
                                      .blended(self.color)
                                      .unwrap();
        let (regular_w,regular_h) = regular_surface.size();
        let mut surface = Surface::new(regular_w + outline_width * 2,
                                       regular_h + outline_width * 2,
                                       ARGB8888).expect("Failed to create new Surface");
        match self.outline {
            Some(outline_color) => {
                // blit the surface containing the border
                let outline_surface = font_set.get_outline_font()
                                              .render(self.text.as_str())
                                              .blended(outline_color)
                                              .unwrap();
                outline_surface.blit(None,
                                     surface.deref_mut(),
                                     None);
            },
            None => {}
        };
        regular_surface.blit(None,
                             surface.deref_mut(),
                             Some(Rect::new(outline_width as i32,
                                            outline_width as i32,
                                            regular_w,
                                            regular_h)));
        let mut texture = renderer.create_texture_from_surface(surface).expect("Failed to create Texture from Surface   ");
        texture.set_blend_mode(BlendMode::Blend);
        texture.set_alpha_mod(match self.color {
            Color::RGB(_, _, _) => 255,
            Color::RGBA(_, _, _, a) => a,
        });
        renderer.copy(&texture,None,Some(Rect::new(
                                            origin.0,
                                            origin.1,
                                            regular_w + outline_width * 2,
                                            regular_h + outline_width * 2
                                        )));
        // returns center bottom of syllable
        Rect::new(origin.0,origin.1,outline_width * 2 + regular_w, outline_width * 2 + regular_h)
    }
}
