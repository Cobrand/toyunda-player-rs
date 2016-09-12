use super::Color;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Outline {
    None,
    Light(Color),
    Bold(Color)
}
