mod panel;
mod stack_panel;
mod tab_panel;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyView, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled, View, ViewContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

actions!(dock, [ToggleZoom, ClosePanel]);

/// The main area of the dock.
pub struct DockArea {
    id: SharedString,
    root: View<StackPanel>,
    zoom_view: Option<AnyView>,
}

impl DockArea {
    pub fn new(
        id: impl Into<SharedString>,
        root: View<StackPanel>,
        _cx: &mut ViewContext<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            root,
            zoom_view: None,
        }
    }

    /// Returns the ID of the dock area.
    pub fn id(&self) -> SharedString {
        self.id.clone()
    }

    pub fn set_zoomed_in<P: Panel>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>) {
        self.zoom_view = Some(panel.into());
        cx.notify();
    }

    pub fn set_zoomed_out(&mut self, cx: &mut ViewContext<Self>) {
        self.zoom_view = None;
        cx.notify();
    }

    /// Returns the root stack panel.
    pub fn root(&self) -> View<StackPanel> {
        self.root.clone()
    }
}

impl Render for DockArea {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        // println!("Rendering dock area");
        div()
            .id("dock-area")
            .size_full()
            .overflow_hidden()
            .map(|this| {
                if let Some(zoom_view) = self.zoom_view.clone() {
                    this.child(zoom_view)
                } else {
                    this.child(self.root.clone())
                }
            })
    }
}
