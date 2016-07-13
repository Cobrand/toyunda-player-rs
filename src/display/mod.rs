use displayer::Displayer;

mod misc;
mod text_element;
mod text2d;
pub use self::misc::*;
pub use self::text_element::*;
pub use self::text2d::*;

pub trait Display {
    fn draw(self, &mut Displayer);
}
