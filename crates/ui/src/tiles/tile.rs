use std::{collections::HashMap, sync::Arc};

use crate::{button::Button, popup_menu::PopupMenu};
use gpui::{
    AnyElement, AnyView, AppContext, EventEmitter, FocusHandle, FocusableView, Global, Hsla,
    IntoElement, SharedString, View, WeakView, WindowContext,
};

use rust_i18n::t;

use super::{CanvasArea, CanvasItemInfo, CanvasItemState};

pub enum TileEvent {
    ZoomIn,
    ZoomOut,
    LayoutChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileStyle {
    Default,
    TabBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleStyle {
    pub background: Hsla,
    pub foreground: Hsla,
}

pub trait Tile: EventEmitter<TileEvent> + FocusableView {
    fn tile_name(&self) -> &'static str;
    fn title(&self, _cx: &WindowContext) -> AnyElement {
        SharedString::from(t!("Canvas.Unnamed")).into_any_element()
    }
    fn title_style(&self, _cx: &WindowContext) -> Option<TitleStyle> {
        None
    }
    fn closeable(&self, _cx: &WindowContext) -> bool {
        true
    }
    fn zoomable(&self, _cx: &WindowContext) -> bool {
        true
    }
    fn popup_menu(&self, this: PopupMenu, _cx: &WindowContext) -> PopupMenu {
        this
    }
    fn toolbar_buttons(&self, _cx: &WindowContext) -> Vec<Button> {
        vec![]
    }
    fn dump(&self, _cx: &AppContext) -> CanvasItemState {
        CanvasItemState::new(self)
    }
}

pub trait TileView: 'static + Send + Sync {
    fn tile_name(&self, _cx: &WindowContext) -> &'static str;
    fn title(&self, _cx: &WindowContext) -> AnyElement;
    fn title_style(&self, _cx: &WindowContext) -> Option<TitleStyle>;
    fn closeable(&self, cx: &WindowContext) -> bool;
    fn zoomable(&self, cx: &WindowContext) -> bool;
    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu;
    fn toolbar_buttons(&self, cx: &WindowContext) -> Vec<Button>;
    fn view(&self) -> AnyView;
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle;
    fn dump(&self, cx: &AppContext) -> CanvasItemState;
}

impl<T: Tile> TileView for View<T> {
    fn tile_name(&self, cx: &WindowContext) -> &'static str {
        self.read(cx).tile_name()
    }

    fn title(&self, cx: &WindowContext) -> AnyElement {
        self.read(cx).title(cx)
    }

    fn title_style(&self, cx: &WindowContext) -> Option<TitleStyle> {
        self.read(cx).title_style(cx)
    }

    fn closeable(&self, cx: &WindowContext) -> bool {
        self.read(cx).closeable(cx)
    }

    fn zoomable(&self, cx: &WindowContext) -> bool {
        self.read(cx).zoomable(cx)
    }

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu {
        self.read(cx).popup_menu(menu, cx)
    }

    fn toolbar_buttons(&self, cx: &WindowContext) -> Vec<Button> {
        self.read(cx).toolbar_buttons(cx)
    }

    fn view(&self) -> AnyView {
        self.clone().into()
    }

    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.read(cx).focus_handle(cx)
    }

    fn dump(&self, cx: &AppContext) -> CanvasItemState {
        self.read(cx).dump(cx)
    }
}

impl From<&dyn TileView> for AnyView {
    fn from(handle: &dyn TileView) -> Self {
        handle.view()
    }
}

impl<T: Tile> From<&dyn TileView> for View<T> {
    fn from(value: &dyn TileView) -> Self {
        value.view().downcast::<T>().unwrap()
    }
}

impl PartialEq for dyn TileView {
    fn eq(&self, other: &Self) -> bool {
        self.view() == other.view()
    }
}

pub struct TileRegistry {
    pub(super) items: HashMap<
        String,
        Arc<
            dyn Fn(
                WeakView<CanvasArea>,
                &CanvasItemState,
                &CanvasItemInfo,
                &mut WindowContext,
            ) -> Box<dyn TileView>,
        >,
    >,
}
impl TileRegistry {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}
impl Global for TileRegistry {}

pub fn register_tile<F>(cx: &mut AppContext, tile_name: &str, deserialize: F)
where
    F: Fn(
            WeakView<CanvasArea>,
            &CanvasItemState,
            &CanvasItemInfo,
            &mut WindowContext,
        ) -> Box<dyn TileView>
        + 'static,
{
    if let None = cx.try_global::<TileRegistry>() {
        cx.set_global(TileRegistry::new());
    }

    cx.global_mut::<TileRegistry>()
        .items
        .insert(tile_name.to_string(), Arc::new(deserialize));
}
