mod scroll_view;
mod scrollable;
mod scrollbar;

use gpui::{AnyElement, AnyView, Div, Element, ElementId};
pub use scroll_view::*;
pub use scrollable::*;
pub use scrollbar::*;

pub trait Scrollable: Element + Sized {
    /// Wraps the element in a ScrollView.
    ///
    /// Current this is only have a vertical scrollbar.
    fn scrollable(self, id: impl Into<ElementId>, view: impl Into<AnyView>) -> ScrollView {
        ScrollView::new(id.into(), view, ScrollbarAxis::Vertical).content(move |_| self)
    }
}

impl Scrollable for AnyElement {}
impl Scrollable for Div {}
