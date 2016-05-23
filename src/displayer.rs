extern crate sdl2;
extern crate sdl2_ttf;
use sdl2::render::{Renderer, TextureQuery, BlendMode};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::vec::Vec;
use std::path::Path;
use font::*;
use std::ops::DerefMut;

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
