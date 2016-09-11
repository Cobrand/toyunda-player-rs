use super::{Color,Outline};

#[derive(Debug)]
pub struct TextSubUnit {
    pub text: String,
    pub color: Color,
    pub outline: Option<Outline>,
    pub shadow: Option<Color>,
    pub attach_logo: bool,
}
