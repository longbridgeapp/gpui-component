mod panel;
mod stack_panel;
mod tab_panel;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyWeakView, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Styled, View, ViewContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

actions!(dock, [ToggleZoom]);

/// The main area of the dock.
pub struct DockArea {
    root: View<StackPanel>,
    zoom_view: Option<AnyWeakView>,
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
            println!("------- Zooming in/out the panel. {:?} -------", panel);
            self.zoom_view = Some(panel.downgrade().into());
        }
        cx.notify();
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
