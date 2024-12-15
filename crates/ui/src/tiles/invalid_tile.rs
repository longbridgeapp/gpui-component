use gpui::{
    AnyElement, AppContext, EventEmitter, FocusHandle, FocusableView, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, WindowContext,
};

use crate::theme::ActiveTheme as _;

use super::{CanvasItemState, Tile, TileEvent};

pub(crate) struct InvalidTile {
    name: SharedString,
    focus_handle: FocusHandle,
    old_state: CanvasItemState,
}

impl InvalidTile {
    pub(crate) fn new(name: &str, state: CanvasItemState, cx: &mut WindowContext) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            name: SharedString::from(name.to_owned()),
            old_state: state,
        }
    }
}
impl Tile for InvalidTile {
    fn tile_name(&self) -> &'static str {
        "InvalidTile"
    }

    fn dump(&self, _cx: &AppContext) -> super::CanvasItemState {
        self.old_state.clone()
    }
}
impl EventEmitter<TileEvent> for InvalidTile {}
impl FocusableView for InvalidTile {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl crate::dock::Panel for InvalidTile {
    fn panel_name(&self) -> &'static str {
        "InvalidTile"
    }

    fn title(&self, _cx: &WindowContext) -> AnyElement {
        format!("Invalid Tile: {}", self.name).into_any_element()
    }

    fn closeable(&self, _cx: &WindowContext) -> bool {
        true
    }

    fn zoomable(&self, _cx: &WindowContext) -> bool {
        false
    }

    fn dump(&self, _cx: &AppContext) -> crate::dock::DockItemState {
        crate::dock::DockItemState {
            panel_name: self.old_state.tile_name.clone(),
            children: vec![],
            info: crate::dock::DockItemInfo::Panel(serde_json::Value::Null),
        }
    }
}
impl EventEmitter<crate::dock::PanelEvent> for InvalidTile {}
impl Render for InvalidTile {
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
                "The `{}` tile type is not registered in TileRegistry.",
                self.name.clone()
            ))
    }
}
