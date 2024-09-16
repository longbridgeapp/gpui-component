mod invalid_panel;
mod panel;
mod stack_panel;
mod tab_panel;

use std::sync::Arc;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyElement, AnyView, AppContext, Axis, EventEmitter,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render, SharedString, Styled,
    View, ViewContext, VisualContext, WeakView, WindowContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

pub fn init(cx: &mut AppContext) {
    cx.set_global(PanelRegistry::new());
}

actions!(dock, [ToggleZoom, ClosePanel]);

pub enum DockEvent {
    /// The layout of the dock has changed, subscribers this to save the layout.
    ///
    /// This event is emitted when every time the layout of the dock has changed,
    /// So it emits may be too frequently, you may want to debounce the event.
    LayoutChanged,
}

/// The main area of the dock.
pub struct DockArea {
    id: SharedString,
    items: DockItem,
    zoom_view: Option<AnyView>,
}

/// DockItem is a tree structure that represents the layout of the dock.
#[derive(Clone)]
pub enum DockItem {
    Split {
        axis: gpui::Axis,
        items: Vec<DockItem>,
        sizes: Vec<Option<Pixels>>,
        view: View<StackPanel>,
    },
    Tabs {
        items: Vec<Arc<dyn PanelView>>,
        active_ix: usize,
        view: View<TabPanel>,
    },
}

impl DockItem {
    /// Create DockItem with split layout, each item of panel have equal size.
    pub fn split(
        axis: Axis,
        items: Vec<DockItem>,
        dock_area: &WeakView<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let sizes = vec![None; items.len()];
        Self::split_with_sizes(axis, items, sizes, dock_area, cx)
    }

    /// Create DockItem with split layout, each item of panel have specified size.
    ///
    /// Please note that the `items` and `sizes` must have the same length.
    /// Set `None` in `sizes` to make the index of panel have auto size.
    pub fn split_with_sizes(
        axis: Axis,
        items: Vec<DockItem>,
        sizes: Vec<Option<Pixels>>,
        dock_area: &WeakView<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let mut items = items;
        let stack_panel = cx.new_view(|cx| {
            let mut stack_panel = StackPanel::new(axis, cx);
            for (i, item) in items.iter_mut().enumerate() {
                let view = item.view();
                let size = sizes.get(i).copied().flatten();
                stack_panel.add_panel(view.clone(), size, dock_area.clone(), cx)
            }

            for (i, item) in items.iter().enumerate() {
                let view = item.view();
                let size = sizes.get(i).copied().flatten();
                stack_panel.add_panel(view.clone(), size, dock_area.clone(), cx)
            }

            stack_panel
        });
        Self::Split {
            axis,
            items,
            sizes,
            view: stack_panel,
        }
    }

    /// Create DockItem with tabs layout, items are displayed as tabs.
    ///
    /// The `active_ix` is the index of the active tab, if `None` the first tab is active.
    pub fn tabs(
        items: Vec<Arc<dyn PanelView>>,
        active_ix: Option<usize>,
        dock_area: &WeakView<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let mut new_items: Vec<Arc<dyn PanelView>> = vec![];
        for item in items.into_iter() {
            new_items.push(item)
        }
        Self::new_tabs(new_items, active_ix, dock_area, cx)
    }

    pub fn tab<P: Panel>(
        item: View<P>,
        dock_area: &WeakView<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new_tabs(vec![Arc::new(item.clone())], None, dock_area, cx)
    }

    fn new_tabs(
        items: Vec<Arc<dyn PanelView>>,
        active_ix: Option<usize>,
        dock_area: &WeakView<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let active_ix = active_ix.unwrap_or(0);
        let tab_panel = cx.new_view(|cx| {
            let mut tab_panel = TabPanel::new(None, dock_area.clone(), cx);
            for item in items.iter() {
                tab_panel.add_panel(item.clone(), cx)
            }
            tab_panel.active_ix = active_ix;
            tab_panel
        });

        Self::Tabs {
            items,
            active_ix,
            view: tab_panel,
        }
    }

    /// Returns the views of the dock item.
    fn view(&self) -> Arc<dyn PanelView> {
        match self {
            Self::Split { view, .. } => Arc::new(view.clone()),
            Self::Tabs { view, .. } => Arc::new(view.clone()),
        }
    }

    /// Find existing panel in the dock item.
    pub fn find_panel(&self, panel: Arc<dyn PanelView>) -> Option<Arc<dyn PanelView>> {
        match self {
            Self::Split { items, .. } => {
                items.iter().find_map(|item| item.find_panel(panel.clone()))
            }
            Self::Tabs { items, .. } => items.iter().find(|item| *item == &panel).cloned(),
        }
    }
}

impl DockArea {
    pub fn new(id: impl Into<SharedString>, cx: &mut WindowContext) -> Self {
        let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Horizontal, cx));
        let dock_item = DockItem::Split {
            axis: Axis::Horizontal,
            items: vec![],
            sizes: vec![],
            view: stack_panel.clone(),
        };

        Self {
            id: id.into(),
            items: dock_item,
            zoom_view: None,
        }
    }

    /// The the DockItem as the root of the dock area.
    #[must_use]
    pub fn set_root(&mut self, item: DockItem, cx: &mut ViewContext<Self>) {
        self.subscribe_item(&item, cx);
        self.items = item;

        cx.notify();
    }

    /// Dump the dock panels layout to DockItemState.
    ///
    /// See also `DockItemState::to_item` for the load DockItem from DockItemState.
    pub fn dump(&self, cx: &AppContext) -> DockItemState {
        let root = self.items.view();
        root.dump(cx)
    }

    /// Subscribe event on the panels
    #[allow(clippy::only_used_in_recursion)]
    fn subscribe_item(&self, item: &DockItem, cx: &mut ViewContext<Self>) {
        match item {
            DockItem::Split { items, view, .. } => {
                for item in items {
                    self.subscribe_item(item, cx);
                }

                cx.subscribe(view, move |_, _, event, cx| match event {
                    PanelEvent::LayoutChanged => cx.emit(DockEvent::LayoutChanged),
                    _ => {}
                })
                .detach();
            }
            DockItem::Tabs { .. } => {
                // We subscribe the tab panel event is in StackPanel insert_panel
            }
        }
    }

    /// Subscribe zoom event on the panel
    pub(crate) fn subscribe_panel<P: Panel>(view: &View<P>, cx: &mut ViewContext<DockArea>) {
        cx.subscribe(view, move |_, panel, event, cx| match event {
            PanelEvent::ZoomIn => {
                let dock_area = cx.view().clone();
                let panel = panel.clone();
                cx.spawn(|_, mut cx| async move {
                    let _ = cx.update(|cx| {
                        let _ = dock_area.update(cx, |dock, cx| {
                            dock.set_zoomed_in(panel, cx);
                            cx.notify();
                        });
                    });
                })
                .detach();
            }
            PanelEvent::ZoomOut => {
                let dock_area = cx.view().clone();
                cx.spawn(|_, mut cx| async move {
                    let _ = cx.update(|cx| {
                        let _ = dock_area.update(cx, |view, cx| view.set_zoomed_out(cx));
                    });
                })
                .detach()
            }
            PanelEvent::LayoutChanged => cx.emit(DockEvent::LayoutChanged),
        })
        .detach();
    }

    /// Returns the ID of the dock area.
    pub fn id(&self) -> SharedString {
        self.id.clone()
    }

    pub fn set_zoomed_in<P: Panel>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>) {
        self.zoom_view = Some(panel.into());
        cx.notify();
    }

    pub fn set_zoomed_out(&mut self, cx: &mut ViewContext<Self>) {
        self.zoom_view = None;
        cx.notify();
    }

    fn render_items(&self, _cx: &mut ViewContext<Self>) -> AnyElement {
        match &self.items {
            DockItem::Split { view, .. } => view.clone().into_any_element(),
            DockItem::Tabs { view, .. } => view.clone().into_any_element(),
        }
    }
}
impl EventEmitter<DockEvent> for DockArea {}
impl Render for DockArea {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // println!("Rendering dock area");
        div()
            .id("dock-area")
            .size_full()
            .overflow_hidden()
            .map(|this| {
                if let Some(zoom_view) = self.zoom_view.clone() {
                    this.child(zoom_view)
                } else {
                    this.child(self.render_items(cx))
                }
            })
    }
}
