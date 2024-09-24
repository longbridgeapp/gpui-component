use std::{collections::HashMap, sync::Arc};

use crate::popup_menu::PopupMenu;
use gpui::{
    AnyElement, AnyView, AppContext, Axis, EventEmitter, FocusHandle, FocusableView, Global, Hsla,
    IntoElement, Pixels, SharedString, View, VisualContext, WeakView, WindowContext,
};
use itertools::Itertools;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use super::{invalid_panel::InvalidPanel, DockArea, DockItem};

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

    /// The title of the panel
    fn title(&self, _cx: &WindowContext) -> AnyElement {
        SharedString::from(t!("Dock.Unnamed")).into_any_element()
    }

    /// The theme of the panel title, default is `None`.
    fn title_style(&self, _cx: &WindowContext) -> Option<TitleStyle> {
        None
    }

    /// Whether the panel can be closed, default is `true`.
    fn closeable(&self, _cx: &WindowContext) -> bool {
        true
    }

    /// Return true if the panel is zoomable, default is `false`.
    fn zoomable(&self, _cx: &WindowContext) -> bool {
        true
    }

    /// Return true if the panel is collapsable, default is `false`.
    fn collapsible(&self, _cx: &WindowContext) -> bool {
        false
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
    fn panel_name(&self, _cx: &WindowContext) -> &'static str;
    fn title(&self, _cx: &WindowContext) -> AnyElement;

    fn title_style(&self, _cx: &WindowContext) -> Option<TitleStyle>;

    fn closeable(&self, cx: &WindowContext) -> bool;
    fn zoomable(&self, cx: &WindowContext) -> bool;
    fn collapsible(&self, cx: &WindowContext) -> bool;

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu;

    fn view(&self) -> AnyView;

    fn focus_handle(&self, cx: &AppContext) -> FocusHandle;

    fn dump(&self, cx: &AppContext) -> DockItemState;
}

impl<T: Panel> PanelView for View<T> {
    fn panel_name(&self, cx: &WindowContext) -> &'static str {
        self.read(cx).panel_name()
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

    fn collapsible(&self, cx: &WindowContext) -> bool {
        self.read(cx).collapsible(cx)
    }

    fn popup_menu(&self, menu: PopupMenu, cx: &WindowContext) -> PopupMenu {
        self.read(cx).popup_menu(menu, cx)
    }

    fn view(&self) -> AnyView {
        self.clone().into()
    }

    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.read(cx).focus_handle(cx)
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
    #[serde(rename = "panel")]
    Panel(serde_json::Value),
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

    pub fn panel(value: serde_json::Value) -> Self {
        Self::Panel(value)
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

impl Default for DockItemState {
    fn default() -> Self {
        Self {
            panel_name: "".to_string(),
            children: Vec::new(),
            info: DockItemInfo::Panel(serde_json::Value::Null),
        }
    }
}

impl DockItemState {
    pub fn new(panel_name: &str) -> Self {
        Self {
            panel_name: panel_name.to_string(),
            ..Default::default()
        }
    }

    pub fn add_child(&mut self, panel: DockItemState) {
        self.children.push(panel);
    }

    pub fn to_item(&self, dock_area: WeakView<DockArea>, cx: &mut WindowContext) -> DockItem {
        let info = self.info.clone();

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
            DockItemInfo::Panel(_) => {
                let view = if let Some(f) = cx
                    .global::<PanelRegistry>()
                    .items
                    .get(&self.panel_name)
                    .cloned()
                {
                    f(dock_area.clone(), info.clone(), cx)
                } else {
                    // Show an invalid panel if the panel is not registered.
                    Box::new(
                        cx.new_view(|cx| InvalidPanel::new(&self.panel_name, info.clone(), cx)),
                    )
                };

                DockItem::tabs(vec![view.into()], None, &dock_area, cx)
            }
        }
    }
}

pub struct PanelRegistry {
    items: HashMap<
        String,
        Arc<dyn Fn(WeakView<DockArea>, DockItemInfo, &mut WindowContext) -> Box<dyn PanelView>>,
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
pub fn register_panel<F>(cx: &mut AppContext, panel_name: &str, deserialize: F)
where
    F: Fn(WeakView<DockArea>, DockItemInfo, &mut WindowContext) -> Box<dyn PanelView> + 'static,
{
    if let None = cx.try_global::<PanelRegistry>() {
        cx.set_global(PanelRegistry::new());
    }

    cx.global_mut::<PanelRegistry>()
        .items
        .insert(panel_name.to_string(), Arc::new(deserialize));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize_item_state() {
        let json = include_str!("../../tests/fixtures/layout.json");
        let state: DockItemState = serde_json::from_str(json).unwrap();
        assert_eq!(state.panel_name, "StackPanel");
        assert_eq!(state.children.len(), 3);
        assert_eq!(state.children[0].panel_name, "StackPanel");
        assert_eq!(state.children[1].children.len(), 2);
        assert_eq!(state.children[1].children[0].panel_name, "TabPanel");
        assert_eq!(state.children[1].panel_name, "StackPanel");
    }
}
