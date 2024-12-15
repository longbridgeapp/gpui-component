use std::sync::Arc;

use crate::{button::Button, popup_menu::PopupMenu};
use gpui::{
    AnyElement, AnyView, AppContext, EventEmitter, FocusHandle, FocusableView, Hsla, IntoElement,
    SharedString, View, WindowContext,
};

use rust_i18n::t;

use super::{CanvasItemInfo, CanvasItemState};

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

/// A shim that implements TileView by delegating to a PanelView
struct PanelViewAsTileView {
    inner: Box<dyn crate::dock::PanelView>,
}

impl TileView for PanelViewAsTileView {
    fn tile_name(&self, cx: &WindowContext) -> &'static str {
        self.inner.panel_name(cx)
    }

    fn title(&self, cx: &WindowContext) -> AnyElement {
        self.inner.title(cx)
    }

    fn title_style(&self, cx: &WindowContext) -> Option<TitleStyle> {
        self.inner.title_style(cx).map(|ts| TitleStyle {
            background: ts.background,
            foreground: ts.foreground,
        })
    }

    fn closeable(&self, cx: &WindowContext) -> bool {
        self.inner.closeable(cx)
    }

    fn zoomable(&self, cx: &WindowContext) -> bool {
        self.inner.zoomable(cx)
    }

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu {
        self.inner.popup_menu(menu, cx)
    }

    fn toolbar_buttons(&self, cx: &WindowContext) -> Vec<Button> {
        self.inner.toolbar_buttons(cx)
    }

    fn view(&self) -> AnyView {
        self.inner.view()
    }

    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.inner.focus_handle(cx)
    }

    fn dump(&self, cx: &AppContext) -> CanvasItemState {
        let dock_state = self.inner.dump(cx);
        let children = dock_state
            .children
            .iter()
            .map(|child| CanvasItemState {
                tile_name: child.panel_name.clone(),
                children: vec![],
                info: match &child.info {
                    crate::dock::DockItemInfo::Panel(value) => CanvasItemInfo::Tile(value.clone()),
                    crate::dock::DockItemInfo::Tabs { active_index } => CanvasItemInfo::Tabs {
                        active_index: *active_index,
                    },
                    crate::dock::DockItemInfo::Stack { .. } => {
                        CanvasItemInfo::Tile(serde_json::Value::Null)
                    }
                },
            })
            .collect();

        let info = match &dock_state.info {
            crate::dock::DockItemInfo::Panel(value) => CanvasItemInfo::Tile(value.clone()),
            crate::dock::DockItemInfo::Tabs { active_index } => CanvasItemInfo::Tabs {
                active_index: *active_index,
            },
            crate::dock::DockItemInfo::Stack { .. } => {
                CanvasItemInfo::Tile(serde_json::Value::Null)
            }
        };

        CanvasItemState {
            tile_name: dock_state.panel_name.clone(),
            children,
            info,
        }
    }
}

impl From<Box<dyn crate::dock::PanelView>> for Arc<dyn TileView> {
    fn from(p: Box<dyn crate::dock::PanelView>) -> Self {
        Arc::new(PanelViewAsTileView { inner: p })
    }
}
