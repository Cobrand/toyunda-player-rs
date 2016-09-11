use super::{OverlayFrame,Rect};
pub trait Display {
	fn display(&mut self,&OverlayFrame) -> Vec<Rect>;
}
