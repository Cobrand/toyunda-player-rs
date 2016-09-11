use ::utils::{RGB,RGBA};
#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct Color {
	pub red:u8,
	pub green:u8,
	pub blue:u8
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
pub struct AlphaColor {
	pub red:u8,
	pub green:u8,
	pub blue:u8,
	pub alpha:u8
}

impl RGB for AlphaColor {
	fn r(&self) -> u8 {
		self.red
	}
	fn g(&self) -> u8 {
		self.green
	}
	fn b(&self) -> u8 {
		self.blue
	}
	fn new(r:u8,g:u8,b:u8) -> AlphaColor {
		AlphaColor {
			red:r,
			green:g,
			blue:b,
			alpha:255
		}
	}
}

impl RGBA for AlphaColor {
	fn a(&self) -> u8 {
		self.alpha
	}
	fn new_rgba(r:u8,g:u8,b:u8,a:u8) -> AlphaColor {
		AlphaColor {
			red:r,
			green:g,
			blue:b,
			alpha:a
		}
	}
}

impl From<Color> for AlphaColor {
	fn from(c:Color) -> AlphaColor {
		AlphaColor {
			red:c.red,
			green:c.green,
			blue:c.blue,
			alpha:255
		}
	}
}
