mod panel;
mod stack_panel;
mod tab_panel;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyView, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Styled, View, ViewContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

use crate::theme::ActiveTheme;

actions!(dock, [ToggleZoom]);

/// The main area of the dock.
pub struct DockArea {
    root: View<StackPanel>,
    zoom_view: Option<AnyView>,
}

impl DockArea {
    pub fn new(root: View<StackPanel>, _cx: &mut ViewContext<Self>) -> Self {
        Self {
            root,
            zoom_view: None,
        }
    }

    /// Toggles the zoom view.
    pub fn toggle_zoom<P: Panel>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>) {
        if self.zoom_view.is_some() {
            self.zoom_view = None;
        } else {
            self.zoom_view = Some(panel.into());
        }
        cx.notify();
    }
}

impl Render for DockArea {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("dock-area")
            .flex()
            .flex_grow()
            .flex_shrink()
            .overflow_hidden()
            .map(|this| match self.zoom_view.clone() {
                Some(view) => this.bg(cx.theme().tab_bar).p_3().child(
                    div()
                        .size_full()
                        .border_1()
                        .border_color(cx.theme().border)
                        .shadow_lg()
                        .child(view),
                ),
                None => this.child(self.root.clone()),
            })
    }
}
