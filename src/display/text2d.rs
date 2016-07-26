use std::ops::DerefMut;
use display::*;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
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
                                                  ::sdl2::pixels::PixelFormatEnum::ARGB8888)
                                         .expect("Failed to create Surface with ARGB8888 Format");
            let mut width_offset: i32 = if is_outline_enabled {
                0
            } else {
                ::display::font::OUTLINE_WIDTH as i32
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
                               (::display::font::OUTLINE_WIDTH as i32 * 2);
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
        let target_pos = real_position((window_width,window_height),self.pos,self.anchor,(texture_width,texture_height));
        let target_rect: Rect = Rect::new(target_pos.0,target_pos.1, texture_width, texture_height);
        displayer.sdl_renderer_mut().copy(&target_texture, None, Some(target_rect));
    }
}
