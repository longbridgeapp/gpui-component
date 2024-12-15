use gpui::{AppContext, Bounds, Pixels, View, VisualContext as _, WeakView, WindowContext};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

use crate::dock::PanelRegistry;

use super::{invalid_tile::InvalidTile, Canvas, CanvasArea, CanvasItem, Tile};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasAreaState {
    #[serde(default)]
    pub version: Option<usize>,
    pub center: CanvasItemState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasState {
    tile: CanvasItemState,
    size: Pixels,
    open: bool,
}

impl CanvasState {
    pub fn new(canvas: View<Canvas>, cx: &AppContext) -> Self {
        let canvas = canvas.read(cx);

        Self {
            size: canvas.size,
            open: canvas.open,
            tile: canvas.tile.view().dump(cx),
        }
    }

    pub fn to_canvas(
        &self,
        canvas_area: WeakView<CanvasArea>,
        cx: &mut WindowContext,
    ) -> View<Canvas> {
        let item = self.tile.to_item(canvas_area.clone(), cx);
        cx.new_view(|cx| Canvas::from_state(canvas_area.clone(), self.size, item, self.open, cx))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasItemState {
    pub tile_name: String,
    pub children: Vec<CanvasItemState>,
    pub info: CanvasItemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TileState {
    pub state: CanvasItemState,
    pub bounds: Bounds<Pixels>,
    pub z_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanvasItemInfo {
    #[serde(rename = "tabs")]
    Tabs { active_index: usize },
    #[serde(rename = "tile")]
    Tile(serde_json::Value),
    #[serde(rename = "tiles")]
    Tiles(Vec<TileState>),
}

impl CanvasItemInfo {
    pub fn tabs(active_index: usize) -> Self {
        Self::Tabs { active_index }
    }

    pub fn tile(value: serde_json::Value) -> Self {
        Self::Tile(value)
    }

    pub fn tiles(tiles: Vec<TileState>) -> Self {
        Self::Tiles(tiles)
    }

    pub fn active_index(&self) -> Option<usize> {
        match self {
            Self::Tabs { active_index } => Some(*active_index),
            _ => None,
        }
    }
}

impl Default for CanvasItemState {
    fn default() -> Self {
        Self {
            tile_name: "".to_string(),
            children: Vec::new(),
            info: CanvasItemInfo::Tile(serde_json::Value::Null),
        }
    }
}

impl CanvasItemState {
    pub fn new<P: Tile>(tile: &P) -> Self {
        Self {
            tile_name: tile.tile_name().to_string(),
            ..Default::default()
        }
    }

    pub fn add_child(&mut self, tile: CanvasItemState) {
        self.children.push(tile);
    }

    pub fn to_item(&self, canvas_area: WeakView<CanvasArea>, cx: &mut WindowContext) -> CanvasItem {
        let info = self.info.clone();

        let items: Vec<CanvasItem> = self
            .children
            .iter()
            .map(|child| child.to_item(canvas_area.clone(), cx))
            .collect();

        match info {
            CanvasItemInfo::Tabs { active_index } => {
                if items.len() == 1 {
                    return items[0].clone();
                }

                let items = items
                    .iter()
                    .flat_map(|item| match item {
                        CanvasItem::Tabs { items, .. } => items.clone(),
                        _ => {
                            unreachable!("Invalid CanvasItem type in CanvasItemInfo::Tabs")
                        }
                    })
                    .collect_vec();

                CanvasItem::tabs(items, Some(active_index), &canvas_area, cx)
            }
            CanvasItemInfo::Tile(_) => {
                let view = if let Some(f) = cx
                    .global::<PanelRegistry>()
                    .items
                    .get(&self.tile_name)
                    .cloned()
                {
                    f(&canvas_item_info_to_dock_item_info(&info), cx)
                } else {
                    let invalid_tile_view =
                        cx.new_view(|cx| InvalidTile::new(&self.tile_name, self.clone(), cx));
                    let panel_view_box: Box<dyn crate::dock::PanelView> =
                        Box::new(invalid_tile_view);
                    panel_view_box
                };

                CanvasItem::tabs(vec![view.into()], None, &canvas_area, cx)
            }
            CanvasItemInfo::Tiles(state) => {
                let tiles_items = state
                    .iter()
                    .map(|tile_layout| {
                        let item = tile_layout.state.to_item(canvas_area.clone(), cx);
                        (item, tile_layout.bounds, tile_layout.z_index)
                    })
                    .collect();
                CanvasItem::tiles_with_sizes(tiles_items, &canvas_area, cx)
            }
        }
    }
}

fn canvas_item_info_to_dock_item_info(info: &CanvasItemInfo) -> crate::dock::DockItemInfo {
    match info {
        CanvasItemInfo::Tabs { active_index } => crate::dock::DockItemInfo::Tabs {
            active_index: *active_index,
        },
        CanvasItemInfo::Tile(value) => crate::dock::DockItemInfo::Panel(value.clone()),
        CanvasItemInfo::Tiles(_) => crate::dock::DockItemInfo::Panel(serde_json::Value::Null),
    }
}
