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

        let is_outline_enabled = self.text
                                     .iter()
                                     .any(|text_element| text_element.outline.is_some());
        let all_text = self.to_string();
        let font_set = displayer.fonts
                                .get_fittest_font_set(all_text.as_str(),
                                                      (fit_width, fit_height),
                                                      is_outline_enabled)
                                .unwrap();
        let (text_width,text_height) = font_set.get_outline_font()
                                                .size_of(all_text.as_str())
                                                .expect("Unable to get outline pixel size of str");
        let (text_pos_x,text_pos_y) = real_position((window_width,window_height),self.pos,self.anchor,(text_width,text_height));
        let mut width_offset : u32 = 0 ;
        for text_element in self.text.iter() {
            // for each text element, blit it over
            let syllable_rect =
                text_element.blit(font_set,
                                  &mut displayer.renderer,
                                  (text_pos_x + width_offset as i32,text_pos_y));
            if text_element.attach_logo {
                let (syllable_center_x,_) = syllable_rect.center().into();
                let syllable_bottom = syllable_rect.bottom();
                let syllable_height = syllable_rect.height();
                let logo_height = syllable_height * 70 / 100 ;
                match displayer.lyrics_logo {
                    Some(ref texture) => {
                        displayer.renderer.copy(&texture,
                                                None,
                                                Some(Rect::new(syllable_center_x - (logo_height/2) as i32,
                                                               syllable_bottom,
                                                               logo_height,
                                                               logo_height)));
                    },
                    None => {}
                }
                ;
            }
            let (w,h):(u32,u32) = font_set.get_regular_font().size_of(text_element.text.as_str()).unwrap();
            width_offset = width_offset + w ;
        }
    }
}
