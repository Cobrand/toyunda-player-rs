#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct Color {
	pub red:u8,
	pub green:u8,
	pub blue:u8
}

#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
pub struct AlphaColor {
	pub red:u8,
	pub green:u8,
	pub blue:u8,
	pub aalpha:u8
}
