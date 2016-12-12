use super::TextUnit;

#[derive(Debug)]
pub struct OverlayFrame {
    pub text_units: Vec<TextUnit>,
}

impl OverlayFrame {
    pub fn new() -> Self {
        OverlayFrame { text_units: vec![] }
    }
}
