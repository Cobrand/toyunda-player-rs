use super::{OverlayFrame,Rect};
pub trait Display {
	fn display(&self,&OverlayFrame) -> Vec<Rect>;
}
