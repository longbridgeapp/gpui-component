use std::collections::HashMap;

use crate::popup_menu::PopupMenu;
use gpui::{
    AnyView, AppContext, Axis, EventEmitter, FocusableView, Global, Hsla, Pixels, SharedString,
    View, WeakView, WindowContext,
};
use itertools::Itertools;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use super::{DockArea, DockItem};

pub enum PanelEvent {
    ZoomIn,
    ZoomOut,
    LayoutChanged,
}

pub struct TitleStyle {
    pub background: Hsla,
    pub foreground: Hsla,
}

pub trait Panel: EventEmitter<PanelEvent> + FocusableView {
    /// The name of the panel used to serialize, deserialize and identify the panel.
    ///
    /// This is used to identify the panel when deserializing the panel.
    /// Once you have defined a panel name, this must not be changed.
    fn panel_name(&self) -> &'static str;

    /// The title of the panel, default is `None`.
    fn title(&self, _cx: &WindowContext) -> SharedString {
        t!("Dock.Unnamed").into()
    }

    /// The theme of the panel title, default is `None`.
    fn title_style(&self, _cx: &WindowContext) -> Option<TitleStyle> {
        None
    }

    /// Whether the panel can be closed, default is `true`.
    fn closeable(&self, _cx: &WindowContext) -> bool {
        true
    }

    /// The addition popup menu of the panel, default is `None`.
    fn popup_menu(&self, this: PopupMenu, _cx: &WindowContext) -> PopupMenu {
        this
    }

    /// Dump the panel, used to serialize the panel.
    fn dump(&self, _cx: &AppContext) -> DockItemState {
        DockItemState::new(self.panel_name())
    }
}

pub trait PanelView: 'static + Send + Sync {
    fn title(&self, _cx: &WindowContext) -> SharedString;

    fn title_style(&self, _cx: &WindowContext) -> Option<TitleStyle>;

    fn closeable(&self, cx: &WindowContext) -> bool;

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu;

    fn view(&self) -> AnyView;

    fn dump(&self, cx: &AppContext) -> DockItemState;
}

impl<T: Panel> PanelView for View<T> {
    fn title(&self, cx: &WindowContext) -> SharedString {
        self.read(cx).title(cx)
    }

    fn title_style(&self, cx: &WindowContext) -> Option<TitleStyle> {
        self.read(cx).title_style(cx)
    }

    fn closeable(&self, cx: &WindowContext) -> bool {
        self.read(cx).closeable(cx)
    }

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu {
        self.read(cx).popup_menu(menu, cx)
    }

    fn view(&self) -> AnyView {
        self.clone().into()
    }

    fn dump(&self, cx: &AppContext) -> DockItemState {
        self.read(cx).dump(cx)
    }
}

impl From<&dyn PanelView> for AnyView {
    fn from(handle: &dyn PanelView) -> Self {
        handle.view()
    }
}

impl<T: Panel> From<&dyn PanelView> for View<T> {
    fn from(value: &dyn PanelView) -> Self {
        value.view().downcast::<T>().unwrap()
    }
}

impl PartialEq for dyn PanelView {
    fn eq(&self, other: &Self) -> bool {
        self.view() == other.view()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DockItemState {
    pub panel_name: String,
    pub children: Vec<DockItemState>,
    pub info: DockItemInfo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DockItemInfo {
    #[serde(rename = "stack")]
    Stack {
        sizes: Vec<Pixels>,
        /// The axis of the stack, 0 is horizontal, 1 is vertical
        axis: usize,
    },
    #[serde(rename = "tabs")]
    Tabs { active_index: usize },
    #[serde(rename = "custom")]
    Custom(serde_json::Value),
}

impl DockItemInfo {
    pub fn stack(sizes: Vec<Pixels>, axis: Axis) -> Self {
        Self::Stack {
            sizes,
            axis: if axis == Axis::Horizontal { 0 } else { 1 },
        }
    }

    pub fn tabs(active_index: usize) -> Self {
        Self::Tabs { active_index }
    }

    pub fn custom(value: serde_json::Value) -> Self {
        Self::Custom(value)
    }

    pub fn axis(&self) -> Option<Axis> {
        match self {
            Self::Stack { axis, .. } => Some(if *axis == 0 {
                Axis::Horizontal
            } else {
                Axis::Vertical
            }),
            _ => None,
        }
    }

    pub fn sizes(&self) -> Option<&Vec<Pixels>> {
        match self {
            Self::Stack { sizes, .. } => Some(sizes),
            _ => None,
        }
    }

    pub fn active_index(&self) -> Option<usize> {
        match self {
            Self::Tabs { active_index } => Some(*active_index),
            _ => None,
        }
    }
}

impl DockItemState {
    pub fn new(panel_name: &str) -> Self {
        Self {
            panel_name: panel_name.to_string(),
            children: Vec::new(),
            info: DockItemInfo::Tabs { active_index: 0 },
        }
    }

    pub fn add_child(&mut self, panel: DockItemState) {
        self.children.push(panel);
    }

    pub fn to_item(&self, dock_area: WeakView<DockArea>, cx: &mut WindowContext) -> DockItem {
        // TODO: Use the empty panel if the panel is not registered, for the compatibility.

        let info = self.info.clone();
        let f = *cx
            .global::<PanelRegistry>()
            .items
            .get(&self.panel_name)
            .unwrap_or_else(|| {
                panic!(
                    "The {} panel type is not registed in PanelRegistry.",
                    self.panel_name
                )
            });

        let items: Vec<DockItem> = self
            .children
            .iter()
            .map(|child| child.to_item(dock_area.clone(), cx))
            .collect();

        match info {
            DockItemInfo::Stack { sizes, axis } => {
                let axis = if axis == 0 {
                    Axis::Horizontal
                } else {
                    Axis::Vertical
                };
                let sizes = sizes.iter().map(|s| Some(*s)).collect_vec();
                DockItem::split_with_sizes(axis, items, sizes, &dock_area, cx)
            }
            DockItemInfo::Tabs { active_index } => {
                if items.len() == 1 {
                    return items[0].clone();
                }

                let items = items
                    .iter()
                    .flat_map(|item| match item {
                        DockItem::Tabs { items, .. } => items.clone(),
                        _ => {
                            unreachable!("Invalid DockItem type in DockItemInfo::Tabs")
                        }
                    })
                    .collect_vec();

                DockItem::tabs(items, Some(active_index), &dock_area, cx)
            }
            DockItemInfo::Custom(_) => {
                let view = f(dock_area.clone(), info.clone(), cx);
                DockItem::tabs(vec![view.into()], None, &dock_area, cx)
            }
        }
    }
}

pub struct PanelRegistry {
    items: HashMap<
        String,
        fn(WeakView<DockArea>, DockItemInfo, &mut WindowContext) -> Box<dyn PanelView>,
    >,
}
impl PanelRegistry {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}
impl Global for PanelRegistry {}

/// Register the Panel init by panel_name to global registry.
pub fn register_panel(
    cx: &mut AppContext,
    panel_name: &str,
    deserialize: fn(WeakView<DockArea>, DockItemInfo, &mut WindowContext) -> Box<dyn PanelView>,
) {
    if let None = cx.try_global::<PanelRegistry>() {
        cx.set_global(PanelRegistry::new());
    }

    cx.global_mut::<PanelRegistry>()
        .items
        .insert(panel_name.to_string(), deserialize);
}
