use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, rems, AnchorCorner, AppContext, DismissEvent, Empty,
    EventEmitter, FocusHandle, FocusableView, InteractiveElement as _, IntoElement, ParentElement,
    Render, ScrollHandle, StatefulInteractiveElement, Styled, ViewContext, WeakView, WindowContext,
};
use rust_i18n::t;

use crate::{
    button::{Button, ButtonVariants as _},
    h_flex,
    popup_menu::{PopupMenu, PopupMenuExt},
    tab::{Tab, TabBar},
    theme::ActiveTheme,
    tiles::CanvasItemInfo,
    v_flex, IconName, Selectable, Sizable,
};

use super::{
    CanvasArea, CanvasItemState, CloseTile, Tile, TileEvent, TileStyle, TileView, ToggleZoom,
};

#[derive(Clone, Copy)]
struct TabState {
    closeable: bool,
    zoomable: bool,
}

pub struct TabTile {
    focus_handle: FocusHandle,
    canvas_area: WeakView<CanvasArea>,
    pub(crate) tiles: Vec<Arc<dyn TileView>>,
    pub(crate) active_ix: usize,
    pub(crate) closeable: bool,
    tab_bar_scroll_handle: ScrollHandle,
    is_zoomed: bool,
    is_collapsed: bool,
}

impl Tile for TabTile {
    fn tile_name(&self) -> &'static str {
        "TabTile"
    }

    fn title(&self, cx: &WindowContext) -> gpui::AnyElement {
        self.active_tile()
            .map(|tile| tile.title(cx))
            .unwrap_or("Empty Tab".into_any_element())
    }

    fn closeable(&self, cx: &WindowContext) -> bool {
        if !self.closeable {
            return false;
        }

        self.active_tile()
            .map(|tile| tile.closeable(cx))
            .unwrap_or(false)
    }

    fn zoomable(&self, cx: &WindowContext) -> bool {
        self.active_tile()
            .map(|tile| tile.zoomable(cx))
            .unwrap_or(false)
    }

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu {
        if let Some(tile) = self.active_tile() {
            tile.popup_menu(menu, cx)
        } else {
            menu
        }
    }

    fn toolbar_buttons(&self, cx: &WindowContext) -> Vec<Button> {
        if let Some(tile) = self.active_tile() {
            tile.toolbar_buttons(cx)
        } else {
            vec![]
        }
    }

    fn dump(&self, cx: &AppContext) -> CanvasItemState {
        let mut state = CanvasItemState::new(self);
        for tile in self.tiles.iter() {
            state.add_child(tile.dump(cx));
            state.info = CanvasItemInfo::tabs(self.active_ix);
        }
        state
    }
}

impl TabTile {
    pub fn new(canvas_area: WeakView<CanvasArea>, cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            canvas_area,
            tiles: Vec::new(),
            active_ix: 0,
            tab_bar_scroll_handle: ScrollHandle::new(),
            is_zoomed: false,
            is_collapsed: false,
            closeable: true,
        }
    }

    pub fn active_tile(&self) -> Option<Arc<dyn TileView>> {
        self.tiles.get(self.active_ix).cloned()
    }

    fn set_active_ix(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        self.active_ix = ix;
        self.tab_bar_scroll_handle.scroll_to_item(ix);
        self.focus_active_tile(cx);
        cx.emit(TileEvent::LayoutChanged);
        cx.notify();
    }

    pub fn add_tile(&mut self, tile: Arc<dyn TileView>, cx: &mut ViewContext<Self>) {
        self.add_tile_with_active(tile, true, cx);
    }

    fn add_tile_with_active(
        &mut self,
        tile: Arc<dyn TileView>,
        active: bool,
        cx: &mut ViewContext<Self>,
    ) {
        if self
            .tiles
            .iter()
            .any(|p| p.view().entity_id() == tile.view().entity_id())
        {
            return;
        }

        self.tiles.push(tile);
        if active {
            self.set_active_ix(self.tiles.len() - 1, cx);
        }
        cx.emit(TileEvent::LayoutChanged);
        cx.notify();
    }

    pub(super) fn set_collapsed(&mut self, collapsed: bool, cx: &mut ViewContext<Self>) {
        self.is_collapsed = collapsed;
        cx.notify();
    }

    fn render_toolbar(&self, state: TabState, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_zoomed = self.is_zoomed && state.zoomable;
        let view = cx.view().clone();
        let build_popup_menu = move |this, cx: &WindowContext| view.read(cx).popup_menu(this, cx);

        h_flex()
            .gap_2()
            .occlude()
            .items_center()
            .children(
                self.toolbar_buttons(cx)
                    .into_iter()
                    .map(|btn| btn.xsmall().ghost()),
            )
            .when(self.is_zoomed, |this| {
                this.child(
                    Button::new("zoom")
                        .icon(IconName::Minimize)
                        .xsmall()
                        .ghost()
                        .tooltip(t!("Canvas.Zoom Out"))
                        .on_click(
                            cx.listener(|view, _, cx| view.on_action_toggle_zoom(&ToggleZoom, cx)),
                        ),
                )
            })
            .child(
                Button::new("menu")
                    .icon(IconName::Ellipsis)
                    .xsmall()
                    .ghost()
                    .popup_menu(move |this, cx| {
                        build_popup_menu(this, cx)
                            .when(state.zoomable, |this| {
                                let name = if is_zoomed {
                                    t!("Canvas.Zoom Out")
                                } else {
                                    t!("Canvas.Zoom In")
                                };
                                this.separator().menu(name, Box::new(ToggleZoom))
                            })
                            .when(state.closeable, |this| {
                                this.separator()
                                    .menu(t!("Canvas.Close"), Box::new(CloseTile))
                            })
                    })
                    .anchor(AnchorCorner::TopRight),
            )
    }

    fn render_title_bar(&self, state: TabState, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let Some(canvas_area) = self.canvas_area.upgrade() else {
            return div().into_any_element();
        };
        let tile_style = canvas_area.read(cx).tile_style;

        if self.tiles.len() == 1 && tile_style == TileStyle::Default {
            let tile = self.tiles.get(0).unwrap();
            let title_style = tile.title_style(cx);

            return h_flex()
                .justify_between()
                .items_center()
                .line_height(rems(1.0))
                .h(px(30.))
                .py_2()
                .px_3()
                .when_some(title_style, |this, theme| {
                    this.bg(theme.background).text_color(theme.foreground)
                })
                .child(
                    div()
                        .id("tab")
                        .flex_1()
                        .min_w_16()
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(tile.title(cx)),
                )
                .child(
                    h_flex()
                        .flex_shrink_0()
                        .ml_1()
                        .gap_1()
                        .child(self.render_toolbar(state, cx)),
                )
                .into_any_element();
        }

        TabBar::new("tab-bar")
            .track_scroll(self.tab_bar_scroll_handle.clone())
            .children(self.tiles.iter().enumerate().map(|(ix, tile)| {
                let mut active = ix == self.active_ix;
                let disabled = self.is_collapsed;

                if self.is_collapsed {
                    active = false;
                }

                Tab::new(("tab", ix), tile.title(cx))
                    .py_2()
                    .selected(active)
                    .disabled(disabled)
                    .when(!disabled, |this| {
                        this.on_click(cx.listener(move |view, _, cx| {
                            view.set_active_ix(ix, cx);
                        }))
                    })
            }))
            .child(
                div()
                    .id("tab-bar-empty-space")
                    .h_full()
                    .flex_grow()
                    .min_w_16(),
            )
            .suffix(
                h_flex()
                    .items_center()
                    .top_0()
                    .right_0()
                    .border_l_1()
                    .border_b_1()
                    .h_full()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().tab_bar)
                    .px_2()
                    .gap_1()
                    .child(self.render_toolbar(state, cx)),
            )
            .into_any_element()
    }

    fn render_active_tile(&self) -> impl IntoElement {
        if self.is_collapsed {
            return Empty {}.into_any_element();
        }

        self.active_tile()
            .map(|tile| {
                div()
                    .id("tab-content")
                    .group("")
                    .overflow_y_scroll()
                    .overflow_x_hidden()
                    .flex_1()
                    .child(tile.view())
                    .into_any_element()
            })
            .unwrap_or(Empty {}.into_any_element())
    }

    fn focus_active_tile(&self, cx: &mut ViewContext<Self>) {
        if let Some(active_tile) = self.active_tile() {
            active_tile.focus_handle(cx).focus(cx);
        }
    }

    fn on_action_toggle_zoom(&mut self, _: &ToggleZoom, cx: &mut ViewContext<Self>) {
        if !self.zoomable(cx) {
            return;
        }

        if !self.is_zoomed {
            cx.emit(TileEvent::ZoomIn)
        } else {
            cx.emit(TileEvent::ZoomOut)
        }
        self.is_zoomed = !self.is_zoomed;
    }
}

impl FocusableView for TabTile {
    fn focus_handle(&self, cx: &AppContext) -> gpui::FocusHandle {
        if let Some(active_tile) = self.active_tile() {
            active_tile.focus_handle(cx)
        } else {
            self.focus_handle.clone()
        }
    }
}
impl EventEmitter<DismissEvent> for TabTile {}
impl EventEmitter<TileEvent> for TabTile {}
impl Render for TabTile {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        let focus_handle = self.focus_handle(cx);
        let state = TabState {
            closeable: self.closeable(cx),
            zoomable: self.zoomable(cx),
        };

        v_flex()
            .id("tab-tile")
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::on_action_toggle_zoom))
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().background)
            .child(self.render_title_bar(state, cx))
            .child(self.render_active_tile())
    }
}
