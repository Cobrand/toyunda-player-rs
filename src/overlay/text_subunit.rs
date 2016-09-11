use super::{Color,AlphaColor,Outline};

#[derive(Debug)]
pub struct TextSubUnit {
    pub text: String,
    pub color: AlphaColor,
    pub outline: Outline,
    pub shadow: Option<Color>,
    pub attach_logo: bool,
}
