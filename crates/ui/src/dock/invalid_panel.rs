use gpui::{
    AppContext, EventEmitter, FocusHandle, FocusableView, ParentElement as _, Render, SharedString,
    Styled as _, WindowContext,
};

use crate::theme::ActiveTheme as _;

use super::{Panel, PanelEvent};

pub(crate) struct InvalidPanel {
    name: SharedString,
    focus_handle: FocusHandle,
}

impl InvalidPanel {
    pub(crate) fn new(name: &str, cx: &mut WindowContext) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            name: SharedString::from(name.to_owned()),
        }
    }
}
impl Panel for InvalidPanel {
    fn panel_name(&self) -> &'static str {
        "InvalidPanel"
    }
}
impl EventEmitter<PanelEvent> for InvalidPanel {}
impl FocusableView for InvalidPanel {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl Render for InvalidPanel {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        gpui::div()
            .size_full()
            .my_6()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child(format!(
                "The `{}` panel type is not registed in PanelRegistry.",
                self.name.clone()
            ))
    }
}
