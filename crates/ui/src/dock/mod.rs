mod panel;
mod stack_panel;
mod tab_panel;

use std::sync::Arc;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyElement, AnyView, Axis, InteractiveElement as _,
    IntoElement, ParentElement as _, Pixels, Render, SharedString, Styled, View, ViewContext,
    VisualContext, WindowContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

use crate::Placement;

actions!(dock, [ToggleZoom, ClosePanel]);

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
        dock_area: &View<DockArea>,
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
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let stack_panel = cx.new_view(|cx| {
            let mut stack_panel = StackPanel::new(axis, cx);
            for (i, item) in items.iter().enumerate() {
                let view = item.view();
                let size = sizes.get(i).copied().flatten();
                stack_panel.add_panel(view.clone(), size, dock_area.downgrade(), cx)
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
    pub fn tabs<P: Panel>(
        items: Vec<View<P>>,
        active_ix: Option<usize>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let mut new_items: Vec<Arc<dyn PanelView>> = vec![];
        let active_ix = active_ix.unwrap_or(0);
        let tab_panel = cx.new_view(|cx| {
            let mut tab_panel = TabPanel::new(None, dock_area.downgrade(), cx);

            for item in items.iter() {
                let item = Arc::new(item.clone());
                new_items.push(item.clone());
                tab_panel.add_panel(item.clone(), cx)
            }

            tab_panel
        });

        Self::Tabs {
            items: new_items,
            active_ix,
            view: tab_panel,
        }
    }

    pub fn tab<P: Panel>(
        item: View<P>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::tabs(vec![item], None, dock_area, cx)
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

    /// Find existing dock item in the dock item by panel.
    pub fn find_dock_item(&mut self, panel: Arc<dyn PanelView>) -> Option<&mut DockItem> {
        match self {
            Self::Tabs { view, .. } => {
                if view.view() == panel.view() {
                    Some(self)
                } else {
                    None
                }
            }
            Self::Split { items, .. } => items
                .iter_mut()
                .find_map(|item| item.find_dock_item(panel.clone())),
        }
    }

    /// Add a panel to the dock item.
    pub fn add(
        &mut self,
        panel: Arc<dyn PanelView>,
        ix: Option<usize>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) {
        match self {
            Self::Split {
                items, sizes, view, ..
            } => {
                // let item = Self::tabs(vec![], active_ix, dock_area, cx)
                // if let Some(ix) = ix {
                //     items.insert(ix, item.clone());
                //     sizes.insert(ix, None);
                // } else {
                //     items.push(item.clone());
                //     sizes.push(None);
                // }
                // view.update(cx, |view, cx| {
                //     view.add_panel(item.view(), None, dock_area.downgrade(), cx);
                // })
            }
            Self::Tabs { items, view, .. } => {
                if let Some(ix) = ix {
                    items.insert(ix, panel.clone());
                } else {
                    items.push(panel.clone());
                }

                let view = view.clone();
                cx.defer(move |cx| {
                    view.update(cx, |view, cx| {
                        if let Some(ix) = ix {
                            view.insert_panel_at(panel, ix, cx);
                        } else {
                            view.add_panel(panel, cx);
                        }
                    })
                });
            }
        }
    }

    pub(super) fn split_to(
        &mut self,
        panel: Arc<dyn PanelView>,
        placement: Placement,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) {
        // let item = Self::Panel { view: panel };
        // let new_tab_item = Self::tabs(vec![item], None, dock_area, cx);

        // match self {
        //     Self::Tabs { .. } => {}
        //     _ => {
        //         unreachable!("Only DockItem::Tabs can be split to DockItem::Tabs")
        //     }
        // }
    }

    /// Remove a panel from the dock item.
    pub fn remove(&mut self, panel: Arc<dyn PanelView>, cx: &mut WindowContext) {
        match self {
            Self::Split {
                items, sizes, view, ..
            } => {
                if let Some(ix) = items
                    .iter()
                    .position(|item| item.find_panel(panel.clone()).is_some())
                {
                    items.remove(ix);
                    sizes.remove(ix);
                    view.update(cx, |view, cx| {
                        view.remove_panel(panel.clone(), cx);
                    })
                }
            }
            Self::Tabs { items, view, .. } => {
                if let Some(ix) = items.iter().position(|item| item == &panel) {
                    items.remove(ix);
                    view.update(cx, |view, cx| {
                        view.remove_panel(panel.clone(), cx);
                    })
                }
            }
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

    /// Move the `panel` to the `tab_panel`.
    pub(super) fn move_panel(
        &mut self,
        panel: Arc<dyn PanelView>,
        from_tab_panel: View<TabPanel>,
        to_tab_panel: View<TabPanel>,
        ix: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) {
        let dock_area = cx.view().clone();
        let is_same_tab = from_tab_panel == to_tab_panel;
        if is_same_tab {
            return;
        }

        if let Some(from_item) = self.items.find_dock_item(Arc::new(from_tab_panel)) {
            from_item.remove(panel.clone(), cx);
        } else {
            panic!("TabPanel not found in DockArea");
        }

        if let Some(item) = self.items.find_dock_item(Arc::new(to_tab_panel)) {
            item.add(panel, ix, &dock_area, cx);
        } else {
            panic!("TabPanel not found in DockArea");
        }
    }

    pub(super) fn split_panel(
        &mut self,
        panel: Arc<dyn PanelView>,
        from_tab_panel: View<TabPanel>,
        to_tab_panel: View<TabPanel>,
        placement: Placement,
        cx: &mut ViewContext<Self>,
    ) {
        let dock_area = cx.view().clone();

        if let Some(from_item) = self.items.find_dock_item(Arc::new(from_tab_panel)) {
            from_item.remove(panel.clone(), cx);
        } else {
            panic!("TabPanel not found in DockArea");
        }

        if let Some(item) = self.items.find_dock_item(Arc::new(to_tab_panel)) {
            item.split_to(panel, placement, &dock_area, cx);
        } else {
            panic!("TabPanel not found in DockArea");
        }
    }

    /// Subscribe event on the panels
    #[allow(clippy::only_used_in_recursion)]
    fn subscribe_item(&self, item: &DockItem, cx: &mut ViewContext<Self>) {
        let dock_area = cx.view();

        /// Subscribe zoom event on the panel
        fn subscribe_zoom<P: Panel>(
            view: &View<P>,
            dock_area: View<DockArea>,
            cx: &mut ViewContext<DockArea>,
        ) {
            cx.subscribe(view, move |_, panel, event, cx| match event {
                PanelEvent::ZoomIn => {
                    let dock_area = dock_area.clone();
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
                    let dock_area = dock_area.clone();
                    cx.spawn(|_, mut cx| async move {
                        let _ = cx.update(|cx| {
                            let _ = dock_area.update(cx, |view, cx| view.set_zoomed_out(cx));
                        });
                    })
                    .detach()
                }
            })
            .detach();
        }

        match item {
            DockItem::Split { items, .. } => {
                for item in items {
                    self.subscribe_item(item, cx);
                }
            }
            DockItem::Tabs { view, .. } => {
                // We need, only subscribe to the zoom events on the TabPanel
                // Because we always wrap the DockItem::Panel in a DockItem::Tabs
                subscribe_zoom(view, dock_area.clone(), cx);
            }
        }
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
