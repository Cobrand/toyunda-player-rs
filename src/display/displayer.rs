extern crate sdl2_ttf;
use sdl2::render::{Renderer, BlendMode};
use sdl2::pixels::Color;
use ::display::font::*;

pub struct Displayer<'a> {
    fonts: FontList,
    renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: sdl2_ttf::Sdl2TtfContext,
}

impl<'a> Displayer<'a> {
    pub fn new(mut renderer: Renderer<'a>)
               -> Result<Displayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = sdl2_ttf::init().unwrap();
        let font_list = FontList::new(&ttf_context).unwrap();
        let displayer = Displayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
        };
        Ok(displayer)
    }

    pub fn fatal_error_message(&self,title:&str,info:&str) {
        ::sdl2::messagebox::show_simple_message_box(::sdl2::messagebox::MESSAGEBOX_ERROR,
                                                    title,
                                                    info,
                                                    self.sdl_renderer().window());
    }

    // width and height must be between 0 and 1
    pub fn sub_screen_dims(&self,
                           width: Option<f32>,
                           height: Option<f32>)
                           -> (Option<u32>, Option<u32>) {
        let dims: (u32, u32) = self.renderer.window().unwrap().size();
        (width.and_then(|width| Some((width * (dims.0 as f32)) as u32)),
         height.and_then(|height| Some((height * (dims.1 as f32)) as u32)))
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

    pub fn fonts(&self) -> &FontList {
        &self.fonts
    }
}
