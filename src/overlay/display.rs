use super::{OverlayFrame,Rect};
pub trait Display {
    type Parameters;
	fn display(&mut self,&OverlayFrame,&Self::Parameters) -> Vec<Rect>;
}
