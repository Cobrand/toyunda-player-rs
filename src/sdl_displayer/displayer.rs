use sdl2::render::{Renderer, BlendMode, Texture};
use sdl2_image::{LoadTexture, INIT_PNG, INIT_JPG, init as image_init};
use sdl2_ttf::{Font,Sdl2TtfContext, init as ttf_init};
use sdl2::rect::Rect as SdlRect;
use sdl2::surface::Surface;
use sdl2::pixels::Color as SdlColor;
use utils::fit_dims;

use std::ops::DerefMut;
use ::overlay::*;
use ::overlay::pos::*;
use super::font::*;
use super::*;

impl From<Rect> for SdlRect {
    fn from(r:Rect) -> SdlRect {
        SdlRect::new(r.x,r.y,r.width,r.height)
    }
}

impl From<AlphaColor> for SdlColor {
    fn from(c:AlphaColor) -> SdlColor {
        SdlColor::RGBA(c.red,c.green,c.blue,c.alpha)
    }
}

impl AlphaColor {
    pub fn to_sdl_color(self) -> SdlColor {
        SdlColor::RGBA(self.red,self.green,self.blue,self.alpha)
    }
}

impl Color {
    pub fn to_sdl_color(self) -> SdlColor {
        SdlColor::RGB(self.red,self.green,self.blue)
    }
}

pub struct SDLDisplayer<'a> {
    pub fonts: FontList,
    pub renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: Sdl2TtfContext,
    pub lyrics_logo: Option<Texture>,
}

impl<'a> SDLDisplayer<'a> {
    pub fn new(mut renderer: Renderer<'a>) -> Result<SDLDisplayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = ttf_init().unwrap();
        let font_list = FontList::new(&ttf_context).unwrap();
        let _image_context = image_init(INIT_PNG | INIT_JPG).unwrap();
        // we dont care if imag econtext dies, we only load images once (for now)
        let lyrics_logo: Option<Texture> = match ::std::env::current_exe() {
            Ok(current_exe_path) => {
                match renderer.load_texture(&*current_exe_path.with_file_name("logo_toyunda.png")) {
                    Ok(texture) => Some(texture),
                    Err(e) => {
                        error!("Failed to load logo_toyunda.png : error '{}' ({:?})", e, e);
                        None
                    }
                }
            }
            _ => {
                error!("Failed to open logo_toyunda.png : failed to find current executable");
                None
            }
        };
        let displayer = SDLDisplayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
            lyrics_logo: lyrics_logo,
        };
        Ok(displayer)
    }

    // TODO use this somewhere ?
    #[allow(dead_code)]
    pub fn fatal_error_message(&self, title: &str, info: &str) {
        use ::sdl2::messagebox::ShowMessageError;
        let res =
            ::sdl2::messagebox::show_simple_message_box(::sdl2::messagebox::MESSAGEBOX_ERROR,
                                                        title,
                                                        info,
                                                        self.sdl_renderer().window());
        match res {
            Ok(_) => {}
            Err(ShowMessageError::SdlError(string)) => {
                error!("Unexpected SDL_ERROR {} when creating MessageBox", string);
            }
            Err(_) => error!("Unexpected Error when creating MessageBox"),
        }
    }

    pub fn render(&mut self) {
        self.renderer.window().unwrap().gl_swap_window();
    }

    #[inline]
    pub fn sdl_renderer_mut(&mut self) -> &mut Renderer<'a> {
        &mut self.renderer
    }

    #[inline]
    pub fn sdl_renderer(&self) -> &Renderer<'a> {
        &self.renderer
    }

    fn display_unit(&mut self,text_unit:&TextUnit,params:&SDLDisplayParameters) -> Rect {
        let (offset_x,offset_y) = params.offset.unwrap_or((0,0));
        let (canevas_width, canevas_height) : (u32,u32) = match params.output_size {
            None => self.sdl_renderer().window().unwrap().size(),
            Some(e) => e
        };
        let (fit_width, fit_height) : (Option<u32>,Option<u32>) = match text_unit.size {
            Size::FitPercent(option_x, option_y) => {
                fit_dims((canevas_width,canevas_height),option_x, option_y)
            },
            Size::Fit(x, y) => (x, y),
        };

        let is_outline_enabled = text_unit.text
            .iter()
            .any(|text_element| text_element.outline != Outline::None);
        let all_text = text_unit.to_string();
        let font_set_id = self.fonts
            .get_fittest_font_set_id(all_text.as_str(),
                                     (fit_width, fit_height),
                                     is_outline_enabled)
            .unwrap();
        let (text_width, text_height) = self.fonts.get_font_set(font_set_id).unwrap()
            .get_outline_font()
            .size_of(all_text.as_str())
            .expect("Unable to get outline pixel size of str");
        let (text_pos_x, text_pos_y) = real_position((canevas_width, canevas_height),
                                                     text_unit.pos,
                                                     text_unit.anchor,
                                                     (text_width, text_height));
        let mut width_offset: u32 = 0;
        for text_subunit in text_unit.text.iter() {
            // for each text element, blit it over
            let syllable_rect =
                self.blit_text_subunit(&text_subunit,
                                       font_set_id,
                                       (offset_x + text_pos_x + width_offset as i32,
                                        offset_y + text_pos_y));
            if text_subunit.attach_logo {
                let (syllable_center_x, _) = syllable_rect.center().into();
                let syllable_bottom = syllable_rect.bottom();
                let syllable_height = syllable_rect.height();
                let logo_height = syllable_height * 70 / 100;
                match self.lyrics_logo {
                    Some(ref texture) => {
                        self.renderer.copy(&texture,
                                           None,
                                           Some(SdlRect::new(syllable_center_x -
                                                             (logo_height / 2) as i32,
                                                             syllable_bottom,
                                                             logo_height,
                                                             logo_height)));
                    }
                    None => {}
                };
            };
            let (w, _): (u32, u32) =
                self.fonts.get_font_set(font_set_id).unwrap()
                          .get_regular_font()
                          .size_of(text_subunit.text.as_str()).unwrap_or((0,0));
            width_offset = width_offset + w;
        };
        Rect {
            x:offset_x + text_pos_x,
            y:offset_y + text_pos_y,
            width:text_width,
            height:text_height
        }
    }

    fn blit_text_subunit(&mut self,
                         text_subunit:&TextSubUnit,
                         font_set_id:usize,
                         origin:(i32,i32)) -> SdlRect {
        use ::sdl2::pixels::PixelFormatEnum::ARGB8888;
        fn blit_font_text(dest:&mut Surface,font:&Font,text:&str,color:SdlColor,delta_outline:u32) {
            let (dest_w,dest_h) = dest.size();
            let subdest_rect = SdlRect::new(delta_outline as i32,
                                            delta_outline as i32,
                                            dest_w - (delta_outline * 2),
                                            dest_h - (delta_outline * 2));
            let mut font_surface = font.render(text)
                .blended(color)
                .unwrap();
            let _ = font_surface.set_blend_mode(BlendMode::Blend);
            font_surface.blit(None,dest.deref_mut(),Some(subdest_rect))
                .expect("Failed to blit surface, Display error ?");
        };

        let outline_width = 2u32 ;
        let font_set = self.fonts.get_font_set(font_set_id).unwrap();
        let regular_font = font_set.get_regular_font();
        let light_bold_font = font_set.get_outline_font();
        let bold_font = font_set.get_outline_font();
        let regular_surface = regular_font.render(text_subunit.text.as_str())
            .blended(text_subunit.color)
            .unwrap();
        let (regular_w,regular_h) = regular_surface.size();
        let mut surface = Surface::new(regular_w + outline_width * 2,
                                       regular_h + outline_width * 2,
                                       ARGB8888)
            .expect("Failed to create new Surface");
        let _ = surface.set_blend_mode(BlendMode::Blend);
        match text_subunit.outline {
            Outline::None => {},
            Outline::Light(color) => {
                blit_font_text(&mut surface,
                               light_bold_font,
                               &text_subunit.text,
                               color.to_sdl_color(),
                               0);
            }
            Outline::Bold(color) => {
                blit_font_text(&mut surface,
                               bold_font,
                               &text_subunit.text,
                               color.to_sdl_color(),
                               0);
            }
        };
        blit_font_text(&mut surface,
                       regular_font,
                       &text_subunit.text,
                       text_subunit.color.to_sdl_color(),
                       outline_width);
        let mut texture = self.renderer.create_texture_from_surface(surface)
            .expect("Failed to create Texture from Surface");
        let _ = texture.set_blend_mode(BlendMode::Blend);
        texture.set_alpha_mod(text_subunit.color.alpha);
        let text_rect = SdlRect::new(origin.0,
                                     origin.1,
                                     regular_w + outline_width * 2,
                                     regular_h + outline_width * 2);
        self.renderer.copy(&texture,
                           None,
                           Some(text_rect.clone()));
        text_rect
    }
}

impl<'a> Display for SDLDisplayer<'a> {
    type Parameters = SDLDisplayParameters;
    fn display(&mut self,
               overlay_frame:&OverlayFrame,
               params:&SDLDisplayParameters) -> Vec<Rect> {
        overlay_frame.text_units.iter().map(|text_unit|{
            self.display_unit(text_unit,params)
        }).collect()
    }
}
