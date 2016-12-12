use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use ::overlay::{Color as OverlayColor, Outline as OverlayOutline};
use ::utils::RGB;

impl Deserialize for Color {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor for Visitor {
            type Value = Color;
            fn visit_str<E>(&mut self, value: &str) -> Result<Color, E>
                where E: de::Error
            {
                let mut chars = value.chars();
                if chars.next() == Some('#') {
                    match ::read_color::rgb(&mut chars) {
                        None => Err(E::custom(format!("Color {} is not valid", value))),
                        Some(answer) => {
                            Ok(Color {
                                red: answer[0],
                                green: answer[1],
                                blue: answer[2],
                            })
                        }
                    }
                } else {
                    Err(E::custom(format!("Color must be of the format #RRGGBB; found {}", value)))
                }
            }

            fn visit_map<M>(&mut self, visitor: M) -> Result<Color, M::Error>
                where M: de::MapVisitor
            {
                let mut mvd = de::value::MapVisitorDeserializer::new(visitor);
                let dummy: Result<ColorDummy, _> = Deserialize::deserialize(&mut mvd);
                dummy.map(|dummy| dummy.transform())
            }
        }
        deserializer.deserialize(Visitor)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        serializer.serialize_str(format!("#{:02X}{:02X}{:02X}",self.red,self.green,self.blue).as_str())
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug,Clone,Copy,Deserialize)]
struct ColorDummy {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ColorDummy {
    pub fn transform(self) -> Color {
        Color {
            red: self.red,
            green: self.green,
            blue: self.blue,
        }
    }
}

impl Default for Color {
    fn default() -> Color {
        Color {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

impl From<OverlayColor> for Color {
    fn from(c: OverlayColor) -> Color {
        Color {
            red: c.red,
            green: c.green,
            blue: c.blue,
        }
    }
}

impl From<Color> for OverlayColor {
    fn from(c: Color) -> OverlayColor {
        OverlayColor {
            red: c.red,
            green: c.green,
            blue: c.blue,
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
    fn new(r: u8, g: u8, b: u8) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
        }
    }
}

#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct Outline {
    pub color: Color,
    pub size: u8,
}

impl From<OverlayOutline> for Outline {
    fn from(o: OverlayOutline) -> Outline {
        match o {
            OverlayOutline::None => {
                Outline {
                    color: Color::default(),
                    size: 0,
                }
            }
            OverlayOutline::Light(color) => {
                Outline {
                    color: Color::from(color),
                    size: 1,
                }
            }
            OverlayOutline::Bold(color) => {
                Outline {
                    color: Color::from(color),
                    size: 2,
                }
            } 
        }
    }
}

impl From<Outline> for OverlayOutline {
    fn from(o: Outline) -> OverlayOutline {
        match o.size {
            0 => OverlayOutline::None,
            1 => OverlayOutline::Light(OverlayColor::from(o.color)),
            _ => OverlayOutline::Bold(OverlayColor::from(o.color)),
        }
    }
}
