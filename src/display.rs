extern crate sdl2;
use std::ops::DerefMut;
use displayer::Displayer;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use sdl2::render::{TextureQuery, BlendMode};
use std::vec::Vec;

pub trait Display {
    fn draw(self,&mut Displayer) ;
}

#[derive(Debug)]
pub struct TextElement<'a> {
    pub text:&'a str,
    pub color:Color,
    pub outline:Option<Color>,
    pub shadow:Option<Color>
}

#[derive(Copy,Debug,Clone)]
pub enum Position {
    //Centered,
    TopLeftPosition{x:u32,y:u32},
    //CenterPosition{x:f64,y:f64}
}

#[derive(Copy,Debug,Clone)]
pub enum Size{
    //Percentage(f64),
    Raw(u32),
    FitWidth{width:u32,max_font_size:Option<u32>},
}

#[derive(Debug)]
pub struct Text2D<'a> {
    pub text:Vec<TextElement<'a>>,
    pub size:Size,
    pub position:Position
}

impl<'a> Display for Text2D<'a> {
    fn draw(self,displayer:&mut Displayer) {
        let target_texture = {
            let is_outline_enabled = self.text.iter().any(|text_element|{
                text_element.outline.is_some()
            });
            let window_width = displayer.sdl_renderer().window().unwrap().size().0 ;
            let all_text = self.text.iter().fold(String::default(),|accu_text,text_element|{
                accu_text + text_element.text
            });
            let font_set = displayer.fonts().get_fittest_font_set(all_text.as_str(), window_width as u16,is_outline_enabled).unwrap();
            let (surface_width,surface_height) = if is_outline_enabled {
                font_set.get_outline_font().size_of(all_text.as_str()).expect("Unable to get outline pixel size of str")
            } else {
                font_set.get_regular_font().size_of(all_text.as_str()).expect("Unable to get pixel size of str")
            };
            // ARGB8888 because it's the only one supported on my computer; i hope it's the same everywhere else ?
            let mut target_surface = Surface::new(surface_width,surface_height,sdl2::pixels::PixelFormatEnum::ARGB8888)
                .expect("Failed to create Surface with ARGB8888 Format");

            let mut width_offset : i32 = 0 ;
            for text_element in self.text.iter() {
                let surface = font_set.get_regular_font()
                                      .render(text_element.text)
                                      .blended(Color::RGB(180, 180, 180))
                                      .unwrap();
                let (surface_width,surface_height)=surface.size();
                surface.blit(None,
                             target_surface.deref_mut(),
                             Some(Rect::new(width_offset,0,surface_width,surface_height)));
                width_offset = width_offset + surface_width as i32;
            }
            let target_texture = displayer
                .sdl_renderer()
                .create_texture_from_surface(target_surface)
                .expect("Unable to create empty texture with pixel format ARGB8888");
            target_texture
        };
        let TextureQuery { width:texture_width, height:texture_height, .. } = target_texture.query();
        displayer.sdl_renderer_mut().copy(&target_texture,None,Some(Rect::new(50, 50, texture_width, texture_height)));
    }
}
