use gpui::{Render, RenderOnce};

use crate::Placement;

pub struct Dock {
    placement: Placement,
}

impl Dock {
    pub fn new(placement: Placement) -> Self {
        Self { placement }
    }
}

// pub struct DockArea {
//     top: Option<Dock>,
//     right: Option<Dock>,
//     bottom: Option<Dock>,
//     left: Option<Dock>,
// }

// impl DockArea {
//     pub fn new() -> Self {
//         Self {
//             top: None,
//             right: None,
//             bottom: None,
//             left: None,
//         }
//     }

//     pub fn set_top(&mut self, dock: Dock) {
//         self.top = Some(dock);
//     }

//     pub fn set_right(&mut self, dock: Dock) {
//         self.right = Some(dock);
//     }

//     pub fn set_bottom(&mut self, dock: Dock) {
//         self.bottom = Some(dock);
//     }

//     pub fn set_left(&mut self, dock: Dock) {
//         self.left = Some(dock);
//     }
// }

// impl RenderOnce for DockArea {
//     fn render(self, cx: &mut gpui::WindowContext) -> impl gpui::IntoElement {

//     }
// }
