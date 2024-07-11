use gpui::Axis;

mod panel;
pub use panel::*;

pub fn h_resizable() -> ResizablePanelGroup {
    ResizablePanelGroup::new().axis(Axis::Horizontal)
}

pub fn v_resizable() -> ResizablePanelGroup {
    ResizablePanelGroup::new().axis(Axis::Vertical)
}

pub fn resizable_panel() -> ResizablePanel {
    ResizablePanel::new()
}
