use gpui::{
    div, AppContext, FocusHandle, FocusableView, InteractiveElement, IntoElement, Render,
    ViewContext,
};

/// An invisible element that can hold focus.
pub(crate) struct Empty {
    focus_handle: FocusHandle,
}

impl Empty {
    pub(crate) fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for Empty {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        div().track_focus(&self.focus_handle)
    }
}

impl FocusableView for Empty {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
