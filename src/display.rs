extern crate sdl2;
use displayer::Displayer;
use sdl2::pixels::Color;
use std::vec::Vec;

pub trait Display {
    fn draw(self,&mut Displayer) ;
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
    //Centered,
    TopLeftPosition{x:f64,y:f64},
    //CenterPosition{x:f64,y:f64}
}

enum Size{
    //Percentage(f64),
    Raw(u32),
    //FitWidth{width:u32,max_font_size:Option<u32>},
}

pub struct Text2D<'a> {
    text:Vec<TextElement<'a>>,
    size:Size,
    position:Position
}

impl<'a> Display for Text2D<'a> {
    fn draw(self,displayer:&mut Displayer) {
        
    }
}
