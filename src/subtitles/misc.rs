use serde::de::{self, Deserialize, Deserializer, Error as DeError};
use serde::ser::{Serialize, Serializer};
use ::overlay::{Color as OverlayColor, Outline as OverlayOutline};
use ::utils::RGB;

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(format!("#{:02X}{:02X}{:02X}",self.red,self.green,self.blue).as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonColor {
    Hex(String),
    Struct{
        red: u8,
        green: u8,
        blue: u8,
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
        where D: Deserializer<'de>
    {
        let json_color : JsonColor = JsonColor::deserialize(deserializer)?;
        match json_color {
            JsonColor::Hex(string) => {
                let mut chars_iter = string.trim().chars();
                if chars_iter.next() == Some('#') {
                    match ::read_color::rgb(&mut chars_iter) {
                        None => Err(D::Error::custom(format!("Color {} is not valid", string.trim()))),
                        Some(answer) => {
                            Ok(Color {
                                red: answer[0],
                                green: answer[1],
                                blue: answer[2],
                            })
                        }
                    }
                } else {
                    Err(D::Error::custom(format!("Color must be of the format #RRGGBB; found {}", string.trim())))
                }
            },
            JsonColor::Struct{red, green, blue} => Ok(Color {red, green, blue})
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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
