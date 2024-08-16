use gpui::{FocusableView, ManagedView, Pixels, Render, SharedString};

use crate::Placement;

pub trait Pane: FocusableView {
    fn placement(&self) -> Placement;
    fn title(&self) -> SharedString;
    fn size(&self) -> Pixels;
    fn set_size(&mut self, size: Pixels);
}
