use gpui::{
    AppContext, EventEmitter, FocusHandle, FocusableView, ParentElement as _, Render, SharedString,
    Styled as _, WindowContext,
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
