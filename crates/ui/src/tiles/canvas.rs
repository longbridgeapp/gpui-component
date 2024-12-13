use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder as _, ParentElement as _, Pixels, Render, Styled as _, ViewContext,
    WeakView, WindowContext,
};

use crate::resizable::PANEL_MIN_SIZE;

use super::{CanvasArea, CanvasItem, TileView};

pub struct Canvas {
    pub(crate) tile: CanvasItem,
    pub(super) size: Pixels,
    pub(super) open: bool,
    pub(super) collapsible: bool,
}

impl Canvas {
    pub fn set_collapsible(&mut self, collapsible: bool, cx: &mut ViewContext<Self>) {
        self.collapsible = collapsible;
        if !collapsible {
            self.open = true
        }
        cx.notify();
    }

    pub(super) fn from_state(
        canvas_area: WeakView<CanvasArea>,
        size: Pixels,
        tile: CanvasItem,
        open: bool,
        cx: &mut WindowContext,
    ) -> Self {
        Self::subscribe_tile_events(canvas_area.clone(), &tile, cx);

        if !open {
            match tile.clone() {
                CanvasItem::Tabs { view, .. } => {
                    view.update(cx, |tile, cx| {
                        tile.set_collapsed(true, cx);
                    });
                }
                _ => {}
            }
        }

        Self {
            tile,
            open,
            size,
            collapsible: true,
        }
    }

    fn subscribe_tile_events(
        canvas_area: WeakView<CanvasArea>,
        tile: &CanvasItem,
        cx: &mut WindowContext,
    ) {
        match tile {
            CanvasItem::Tabs { view, .. } => {
                cx.defer({
                    let view = view.clone();
                    move |cx| {
                        _ = canvas_area.update(cx, |this, cx| {
                            this.subscribe_tile(&view, cx);
                        });
                    }
                });
            }
            CanvasItem::Tiles { view, .. } => {
                cx.defer({
                    let view = view.clone();
                    move |cx| {
                        _ = canvas_area.update(cx, |this, cx| {
                            this.subscribe_tile(&view, cx);
                        });
                    }
                });
            }
        }
    }

    pub fn set_tile(&mut self, tile: CanvasItem, cx: &mut ViewContext<Self>) {
        self.tile = tile;
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn toggle_open(&mut self, cx: &mut ViewContext<Self>) {
        self.set_open(!self.open, cx);
    }

    pub fn size(&self) -> Pixels {
        self.size
    }

    pub fn set_size(&mut self, size: Pixels, cx: &mut ViewContext<Self>) {
        self.size = size.max(PANEL_MIN_SIZE);
        cx.notify();
    }

    pub fn set_open(&mut self, open: bool, cx: &mut ViewContext<Self>) {
        self.open = open;
        let item = self.tile.clone();
        cx.defer(move |_, cx| {
            item.set_collapsed(!open, cx);
        });
        cx.notify();
    }

    pub fn add_tile(&mut self, tile: Arc<dyn TileView>, cx: &mut ViewContext<Self>) {
        self.tile.add_tile(tile, cx);
        cx.notify();
    }
}

impl Render for Canvas {
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl gpui::IntoElement {
        if !self.open {
            return div();
        }

        div()
            .relative()
            .overflow_hidden()
            .map(|this| match &self.tile {
                CanvasItem::Tabs { view, .. } => this.child(view.clone()),
                CanvasItem::Tiles { view, .. } => this.child(view.clone()),
            })
    }
}
