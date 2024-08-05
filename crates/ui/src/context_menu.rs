use gpui::{AnchorCorner, AppContext, ElementId, KeyBinding, View, WindowContext};

use crate::popup_menu::PopupMenu;

pub fn init(cx: &mut AppContext) {}

pub struct ContextMenu {
    id: ElementId,
    menu: View<PopupMenu>,
    anchor: AnchorCorner,
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>, cx: &mut WindowContext) -> Self {
        let menu = PopupMenu::build(cx, |cx| {});
        Self {
            id: id.into(),
            menu,
            anchor: AnchorCorner::TopLeft,
        }
    }

    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn menu_item(self, id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        self
    }
}
