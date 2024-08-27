use gpui::{Axis, ViewContext};

mod panel;
pub use panel::*;

pub fn h_resizable(cx: &mut ViewContext<ResizablePanelGroup>) -> ResizablePanelGroup {
    ResizablePanelGroup::new(cx).axis(Axis::Horizontal)
}

pub fn v_resizable(cx: &mut ViewContext<ResizablePanelGroup>) -> ResizablePanelGroup {
    ResizablePanelGroup::new(cx).axis(Axis::Vertical)
}

pub fn resizable_panel() -> ResizablePanel {
    ResizablePanel::new()
}
