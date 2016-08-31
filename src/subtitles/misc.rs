use ::sdl2::pixels::Color as SdlColor;
#[derive(Debug,Clone,Copy,PartialEq,Serialize,Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn to_sdl_color(self) -> SdlColor {
        SdlColor::RGB(self.red, self.green, self.blue)
    }
}
