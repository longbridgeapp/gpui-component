mod panel;
mod stack_panel;
mod tab_panel;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyWeakView, InteractiveElement as _, IntoElement,
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
    zoom_view: Option<AnyWeakView>,
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

    /// Toggles the zoom view.
    pub fn toggle_zoom<P: Panel>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>) {
        if self.zoom_view.is_some() {
            self.zoom_view = None;
        } else {
            self.zoom_view = Some(panel.downgrade().into());
        }
        cx.notify();
    }

    /// Returns the root stack panel.
    pub fn root(&self) -> View<StackPanel> {
        self.root.clone()
    }
}

impl Render for DockArea {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .id("dock-area")
            .size_full()
            .overflow_hidden()
            .map(|this| {
                if let Some(zoom_view) = self.zoom_view.as_ref().and_then(|view| view.upgrade()) {
                    this.child(zoom_view)
                } else {
                    this.child(self.root.clone())
                }
            })
    }
}
