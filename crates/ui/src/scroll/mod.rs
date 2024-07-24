mod scroll_view;
mod scrollable;
mod scrollbar;

use gpui::{AnyElement, AnyView, Div, Element, ElementId, Focusable, InteractiveElement, Stateful};
pub use scroll_view::*;
pub use scrollable::*;
pub use scrollbar::*;

pub trait Scrollable: Element {
    /// Wraps the element in a ScrollView.
    ///
    /// Current this is only have a vertical scrollbar.
    fn scrollable(self, id: impl Into<ElementId>, view: impl Into<AnyView>) -> ScrollView {
        ScrollView::new(
            ElementId::Name(format!("{}:{}", id.into(), "ScrollView").into()),
            view,
            ScrollbarAxis::Vertical,
        )
        .content(move |_| self)
    }
}

impl Scrollable for AnyElement {}
impl Scrollable for Div {}
impl<E> Scrollable for Focusable<E>
where
    E: Element,
    Self: InteractiveElement,
{
}
impl<E> Scrollable for Stateful<E>
where
    E: Element,
    Self: InteractiveElement,
{
}
