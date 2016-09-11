use super::TextSubUnit;
use super::pos::*;
#[derive(Debug)]
pub struct TextUnit {
    pub text: Vec<TextSubUnit>,
    pub size: Size,
    pub pos: (PosX, PosY),
    pub anchor: (f32, f32),
}

impl TextUnit {
    pub fn to_string(&self) -> String {
        self.text.iter().fold(String::default(),
                              |accu_text, text_element| accu_text + &text_element.text)
    }
}
