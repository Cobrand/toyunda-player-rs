extern crate sdl2;
extern crate sdl2_ttf;
use sdl2::render::{Renderer, TextureQuery, BlendMode};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::vec::Vec;
use std::path::Path;
use font::*;
use std::ops::DerefMut;
use display::{self,Display};

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
        let window_width = self.renderer.window().unwrap().size().0 ;
        let font_set = self.fonts.get_fittest_font_set(text, (Some(window_width),None),true).unwrap();
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
        let TextureQuery { width:texture_width, height:texture_height, format:texture_format,.. } = texture.query();
        self.renderer.copy(&mut texture,
                           None,
                           Some(Rect::new(3, 3, texture_width, texture_height)));
    }

    pub fn example(&mut self) {
        let text_element_1 = display::TextElement {
            text:"S",
            color:Color::RGBA(0,0,0,255),
            outline:Some(Color::RGB(255,255,255)),
            shadow:None
        };
        let text_element_2 = display::TextElement {
            text:"L",
            color:Color::RGBA(255,255,255,255),
            outline:Some(Color::RGB(0,0,0)),
            shadow:None
        };
        let text_element_3 = display::TextElement {
            text:"T",
            color:Color::RGBA(255,0,0,200),
            outline:None,
            shadow:None
        };
        let text_2d : display::Text2D = display::Text2D {
            text:vec![text_element_1,text_element_2,text_element_3],
            size:display::Size::FitPercent(Some(0.9),Some(0.1)),
            pos:(display::PosX::Centered,display::PosY::FromTopPercent(0.50)),
            anchor:(0.5,0.5)
        };
        text_2d.draw(self);
    }

    // width and height must be between 0 and 1
    pub fn sub_screen_dims(&self,width:Option<f32>,height:Option<f32>) -> (Option<u32>,Option<u32>){
        let dims : (u32,u32)= self.renderer.window().unwrap().size();
        (
            width.and_then(|width|{
                Some((width *  (dims.0 as f32)) as u32)
            }) ,
            height.and_then(|height|{
                Some((height *  (dims.1 as f32)) as u32)
            })
        )
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
