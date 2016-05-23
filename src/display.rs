extern crate sdl2;
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use std::vec::Vec;

pub trait Display {
    fn draw(self,&mut Renderer) ;
}

struct Outline {
    pub width:u16,
    pub color:Color
}

struct TextElement<'a> {
    text:&'a str,
    color:Color,
    outline:Option<Outline>,
    shadow:Option<Color>
}

enum Position {
    Centered,
    TopLeftPosition{x:f64,y:f64},
    CenterPosition{x:f64,y:f64}
}

pub struct Text2D<'a> {
    text:Vec<TextElement<'a>>,
    size:f64,
}
