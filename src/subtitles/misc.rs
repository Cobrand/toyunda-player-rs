use ::sdl2::pixels::Color as SdlColor;
#[derive(Debug,Clone,Copy,PartialEq,Serialize,Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct Outline {
	pub color:Color
}
