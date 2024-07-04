use gpui::{FocusableView, WindowContext};

/// A trait for focusable elements.
pub trait Focusable: FocusableView {
    fn focus(&self, cx: &mut WindowContext) {
        cx.focus(&self.focus_handle(cx))
    }

    fn is_focused(&self, cx: &WindowContext) -> bool {
        self.focus_handle(cx).is_focused(cx)
    }
}
