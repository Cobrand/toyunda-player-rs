use ::overlay::{Color as OverlayColor,Outline as OverlayOutline};
use ::utils::RGB;
#[derive(Debug,Clone,Copy,PartialEq,Serialize,Deserialize)]

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Default for Color {
    fn default() -> Color {
        Color {red:0,green:0,blue:0}
    }
}

impl From<OverlayColor> for Color {
    fn from(c:OverlayColor) -> Color {
        Color {
            red:c.red,
            green:c.green,
            blue:c.blue
        }
    }
}

impl From<Color> for OverlayColor {
    fn from(c:Color) -> OverlayColor {
        OverlayColor {
            red:c.red,
            green:c.green,
            blue:c.blue
        }
    }
}

impl RGB for Color {
    fn r(&self) -> u8 {
        self.red
    }
    fn g(&self) -> u8 {
        self.green
    }
    fn b(&self) -> u8 {
        self.blue
    }
    fn new(r:u8,g:u8,b:u8) -> Color {
        Color {
            red:r,
            green:g,
            blue:b
        }
    }
}

#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct Outline {
    pub color:Color,
    pub size:u8
}

impl From<OverlayOutline> for Outline {
    fn from(o:OverlayOutline) -> Outline {
        match o {
            OverlayOutline::None => Outline {
                color:Color::default(),
                size:0
            },
            OverlayOutline::Light(color) => Outline {
                color:Color::from(color),
                size:1
            },
            OverlayOutline::Bold(color) => Outline {
                color:Color::from(color),
                size:2
            } 
        }
    }
}

impl From<Outline> for OverlayOutline {
    fn from(o:Outline) -> OverlayOutline {
        match o.size {
            0 => OverlayOutline::None,
            1 => OverlayOutline::Light(OverlayColor::from(o.color)),
            _ => OverlayOutline::Bold(OverlayColor::from(o.color)),
        }
    }
}
