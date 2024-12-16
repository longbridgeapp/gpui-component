mod canvas;
mod invalid_tile;
mod state;
mod tab_tile;
mod tile_canvas;

use anyhow::Result;
pub use canvas::*;
use gpui::{
    actions, canvas, div, prelude::FluentBuilder, AnyElement, AnyView, AppContext, Bounds,
    EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render,
    SharedString, Styled, Subscription, View, ViewContext, VisualContext, WeakView, WindowContext,
};
use std::sync::Arc;
pub use tile_canvas::*;

pub use state::*;
pub use tab_tile::*;

use crate::dock::PanelView;

actions!(canvas, [ToggleZoom, CloseTile]);

pub enum CanvasEvent {
    LayoutChanged,
}

pub struct CanvasArea {
    id: SharedString,
    version: Option<usize>,
    pub(crate) bounds: Bounds<Pixels>,
    items: CanvasItem,
    zoom_view: Option<AnyView>,
    is_locked: bool,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
pub enum CanvasItem {
    Tabs {
        items: Vec<Arc<dyn PanelView>>,
        active_ix: usize,
        view: View<TabTile>,
    },
    Tiles {
        items: Vec<TilesItem>,
        view: View<TileCanvas>,
    },
}

impl std::fmt::Debug for CanvasItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasItem::Tabs {
                items, active_ix, ..
            } => f
                .debug_struct("Tabs")
                .field("items", &items.len())
                .field("active_ix", active_ix)
                .finish(),
            CanvasItem::Tiles { .. } => f.debug_struct("Tiles").finish(),
        }
    }
}

impl CanvasItem {
    pub fn tiles_with_sizes(
        items: Vec<(CanvasItem, Bounds<Pixels>, usize)>,
        canvas_area: &WeakView<CanvasArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let tile_canvas = cx.new_view(|cx| {
            let mut tile_canvas = TileCanvas::new(cx);
            for (canvas_item, bounds, z_index) in items.into_iter() {
                tile_canvas.add_with_z_index(canvas_item.view(), bounds, z_index, cx);
            }
            tile_canvas
        });

        cx.defer({
            let tile_canvas = tile_canvas.clone();
            let canvas_area = canvas_area.clone();
            move |cx| {
                _ = canvas_area.update(cx, |this, cx| {
                    this.subscribe_tile(&tile_canvas, cx);
                });
            }
        });

        Self::Tiles {
            items: tile_canvas.read(cx).tiles.clone(),
            view: tile_canvas,
        }
    }

    pub fn tabs(
        items: Vec<Arc<dyn TileView>>,
        active_ix: Option<usize>,
        canvas_area: &WeakView<CanvasArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let mut new_items: Vec<Arc<dyn TileView>> = vec![];
        for item in items.into_iter() {
            new_items.push(item)
        }
        Self::new_tabs(new_items, active_ix, canvas_area, cx)
    }

    pub fn tab<P: Tile>(
        item: View<P>,
        canvas_area: &WeakView<CanvasArea>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new_tabs(vec![Arc::new(item.clone())], None, canvas_area, cx)
    }

    fn new_tabs(
        items: Vec<Arc<dyn TileView>>,
        active_ix: Option<usize>,
        canvas_area: &WeakView<CanvasArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let active_ix = active_ix.unwrap_or(0);
        let tab_tile = cx.new_view(|cx| {
            let mut tab_tile = TabTile::new(canvas_area.clone(), cx);
            for item in items.iter() {
                tab_tile.add_tile(item.clone(), cx)
            }
            tab_tile.active_ix = active_ix;
            tab_tile
        });

        Self::Tabs {
            items,
            active_ix,
            view: tab_tile,
        }
    }

    pub fn view(&self) -> Arc<dyn TileView> {
        match self {
            Self::Tabs { view, .. } => Arc::new(view.clone()),
            Self::Tiles { view, .. } => Arc::new(view.clone()),
        }
    }

    pub fn find_tile(&self, tile: Arc<dyn TileView>) -> Option<Arc<dyn TileView>> {
        match self {
            Self::Tabs { items, .. } => items.iter().find(|item| *item == &tile).cloned(),
            Self::Tiles { items, .. } => items.iter().find_map(|item| {
                if &item.tile == &tile {
                    Some(item.tile.clone())
                } else {
                    None
                }
            }),
        }
    }

    pub fn add_panel(&mut self, panel: Arc<dyn PanelView>, cx: &mut WindowContext) {
        match self {
            Self::Tabs { view, items, .. } => {
                items.push(panel.clone());
                view.update(cx, |tab_tile, cx| {
                    tab_tile.add_tile(panel, cx);
                });
            }
            Self::Tiles { .. } => {}
        }
    }

    pub fn set_collapsed(&self, collapsed: bool, cx: &mut WindowContext) {
        match self {
            CanvasItem::Tabs { view, .. } => {
                view.update(cx, |tab_tile, cx| {
                    tab_tile.set_collapsed(collapsed, cx);
                });
            }
            CanvasItem::Tiles { .. } => {}
        }
    }
}

impl CanvasArea {
    pub fn new(
        id: impl Into<SharedString>,
        version: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let tile_canvas = cx.new_view(|cx| TileCanvas::new(cx));

        let canvas_item = CanvasItem::Tiles {
            items: vec![],
            view: tile_canvas.clone(),
        };

        let mut this = Self {
            id: id.into(),
            version,
            bounds: Bounds::default(),
            items: canvas_item,
            zoom_view: None,
            is_locked: false,
            _subscriptions: vec![],
        };

        this.subscribe_tile(&tile_canvas, cx);

        this
    }

    pub fn set_version(&mut self, version: usize, cx: &mut ViewContext<Self>) {
        self.version = Some(version);
        cx.notify();
    }

    pub fn set_center(&mut self, item: CanvasItem, cx: &mut ViewContext<Self>) {
        self.items = item;
        cx.notify();
    }

    pub fn set_locked(&mut self, locked: bool, _: &mut WindowContext) {
        self.is_locked = locked;
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn add_panel(&mut self, panel: Arc<dyn PanelView>, cx: &mut ViewContext<Self>) {
        self.items.add_tile(panel, cx);
    }

    pub fn load(&mut self, state: CanvasAreaState, cx: &mut ViewContext<Self>) -> Result<()> {
        self.version = state.version;
        let weak_self = cx.view().downgrade();
        self.items = state.center.to_item(weak_self, cx);
        Ok(())
    }

    pub fn dump(&self, cx: &AppContext) -> CanvasAreaState {
        let root = self.items.view();
        let center = root.dump(cx);

        CanvasAreaState {
            version: self.version,
            center,
        }
    }

    pub(crate) fn subscribe_tile<P: Tile>(
        &mut self,
        view: &View<P>,
        cx: &mut ViewContext<CanvasArea>,
    ) {
        let subscription = cx.subscribe(view, move |_, tile, event, cx| match event {
            TileEvent::ZoomIn => {
                let canvas_area = cx.view().clone();
                let tile = tile.clone();
                cx.spawn(|_, mut cx| async move {
                    let _ = cx.update(|cx| {
                        let _ = canvas_area.update(cx, |canvas, cx| {
                            canvas.set_zoomed_in(tile, cx);
                            cx.notify();
                        });
                    });
                })
                .detach();
            }
            TileEvent::ZoomOut => {
                let canvas_area = cx.view().clone();
                cx.spawn(|_, mut cx| async move {
                    let _ = cx.update(|cx| {
                        let _ = canvas_area.update(cx, |view, cx| view.set_zoomed_out(cx));
                    });
                })
                .detach()
            }
            TileEvent::LayoutChanged => {
                cx.emit(CanvasEvent::LayoutChanged);
            }
        });

        self._subscriptions.push(subscription);
    }

    pub fn id(&self) -> SharedString {
        self.id.clone()
    }

    pub fn set_zoomed_in<P: Tile>(&mut self, tile: View<P>, cx: &mut ViewContext<Self>) {
        self.zoom_view = Some(tile.into());
        cx.notify();
    }

    pub fn set_zoomed_out(&mut self, cx: &mut ViewContext<Self>) {
        self.zoom_view = None;
        cx.notify();
    }

    fn render_items(&self, _cx: &mut ViewContext<Self>) -> AnyElement {
        match &self.items {
            CanvasItem::Tabs { view, .. } => view.clone().into_any_element(),
            CanvasItem::Tiles { view, .. } => view.clone().into_any_element(),
        }
    }
}
impl EventEmitter<CanvasEvent> for CanvasArea {}
impl Render for CanvasArea {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();

        div()
            .id("canvas-area")
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, cx| view.update(cx, |r, _| r.bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .map(|this| {
                if let Some(zoom_view) = self.zoom_view.clone() {
                    this.child(zoom_view)
                } else {
                    this.child(
                        div().flex().flex_row().h_full().child(
                            div().flex().flex_1().flex_col().overflow_hidden().child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .child(self.render_items(cx)),
                            ),
                        ),
                    )
                }
            })
    }
}
