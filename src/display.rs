extern crate sdl2;
use displayer::Displayer;
use sdl2::pixels::Color;
use std::vec::Vec;

pub trait Display {
    fn draw(self,&mut Displayer) ;
}

#[derive(Copy,Debug,Clone)]
pub struct Outline {
    pub width:u16,
    pub color:Color
}

#[derive(Debug)]
pub struct TextElement<'a> {
    pub text:&'a str,
    pub color:Color,
    pub outline:Option<Outline>,
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
        if self.text.iter().all(|text_element|{
            text_element.outline.is_some()
        }) {
            // all the text has an outline
            let window_width = displayer.sdl_renderer().window().unwrap().size().0 ;
            let all_text = self.text.iter().fold(String::default(),|accu_text,text_element|{
                accu_text + text_element.text
            });
            let font_set = displayer.fonts().get_fittest_font_set(all_text.as_str(), window_width as u16,true).unwrap();
            println!("yes");
        }
    }
}
