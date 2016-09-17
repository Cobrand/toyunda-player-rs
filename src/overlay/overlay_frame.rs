use super::TextUnit;

#[derive(Debug)]
pub struct OverlayFrame {
    pub text_units:Vec<TextUnit>
}

impl OverlayFrame {
    pub fn merge(mut self,mut other:Self) -> Self {
        self.text_units.append(&mut other.text_units);
        self
    }

    pub fn new() -> Self {
        OverlayFrame {
            text_units:vec!()
        }
    }
}
