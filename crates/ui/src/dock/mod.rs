mod panel;
mod stack_panel;
mod tab_panel;

use std::sync::Arc;

use gpui::{
    actions, div, prelude::FluentBuilder, AnyElement, AnyView, Axis, Entity,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render, SharedString, Styled,
    View, ViewContext, VisualContext, WindowContext,
};
pub use panel::*;
pub use stack_panel::*;
pub use tab_panel::*;

use crate::{AxisExt, Placement};

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
    Split(DockItemStack),
    Tabs(DockItemTabs),
}

#[derive(Clone)]
pub struct DockItemStack {
    axis: gpui::Axis,
    items: Vec<DockItem>,
    sizes: Vec<Option<Pixels>>,
    view: View<StackPanel>,
}

impl DockItemStack {
    fn add_item(
        &mut self,
        panel: Arc<dyn PanelView>,
        placement: Placement,
        size: Option<Pixels>,
        ix: Option<usize>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) {
        let item = DockItem::new_tab(panel.clone(), dock_area, cx);
        if let Some(ix) = ix {
            self.items.insert(ix, item.clone());
            self.sizes.insert(ix, size);
        } else {
            self.items.push(item.clone());
            self.sizes.push(size);
        }

        let ix = ix.unwrap_or_else(|| self.items.len() - 1);

        let stack_panel = self.view.clone();
        let dock_area = dock_area.downgrade();
        cx.defer(move |cx| {
            stack_panel.update(cx, |view, cx| {
                view.insert_panel_at(item.view().clone(), ix, placement, size, dock_area, cx)
            });
        })
    }

    fn remove_item(&mut self, panel: Arc<dyn PanelView>, cx: &mut WindowContext) {
        if let Some(ix) = self
            .items
            .iter()
            .position(|item| item.find_panel(panel.clone()).is_some())
        {
            self.items.remove(ix);
            self.sizes.remove(ix);

            let view = self.view.clone();
            cx.defer(move |cx| {
                view.update(cx, |view, cx| {
                    view.remove_panel(panel.clone(), cx);
                })
            });
        }
    }

    fn split_item(
        &mut self,
        panel: Arc<dyn PanelView>,
        tab_panel: Arc<dyn PanelView>,
        placement: Placement,
        size: Option<Pixels>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) {
        let ix = self
            .items
            .iter()
            .position(|item| item.find_panel(tab_panel.clone()).is_some());
        let parent_axis = self.axis;

        if parent_axis.is_vertical() && placement.is_vertical() {
            self.add_item(panel, placement, size, ix, dock_area, cx);
        } else if parent_axis.is_horizontal() && placement.is_horizontal() {
            self.add_item(panel, placement, size, ix, dock_area, cx);
        } else {
            // let tab_panel = tab_panel.clone();
            // let stack_panel = self.view.clone();
            // let new_stack_panel = if self.items.len() <= 1 {
            //     self.remove_all_items(cx);
            //     self.axis = placement.axis();
            //     stack_panel
            // } else {
            //     let mut new_stack_item = DockItem::split(placement.axis(), vec![self.clone()], dock_area, cx);
            //     new_stack_item.
            // };
        }
    }

    fn remove_all_items(&mut self, cx: &mut WindowContext) {
        let items = std::mem::take(&mut self.items);
        let sizes = std::mem::take(&mut self.sizes);

        let view = self.view.clone();
        cx.defer(move |cx| {
            view.update(cx, |view, cx| {
                view.remove_all_panels(cx);
            })
        });
    }
}

#[derive(Clone)]
pub struct DockItemTabs {
    items: Vec<Arc<dyn PanelView>>,
    active_ix: usize,
    view: View<TabPanel>,
}

impl DockItemTabs {
    fn add_item(&mut self, panel: Arc<dyn PanelView>, ix: Option<usize>, cx: &mut WindowContext) {
        if let Some(ix) = ix {
            self.items.insert(ix, panel.clone());
        } else {
            self.items.push(panel.clone());
        }

        let view = self.view.clone();
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

    fn remove_item(&mut self, panel: Arc<dyn PanelView>, cx: &mut WindowContext) {
        if let Some(ix) = self.items.iter().position(|item| item == &panel) {
            self.items.remove(ix);
            if self.active_ix == ix {
                self.active_ix = self.active_ix.saturating_sub(1);
            }

            let view = self.view.clone();
            cx.defer(move |cx| {
                view.update(cx, |view, cx| {
                    view.remove_panel(panel.clone(), cx);
                })
            });
        }
    }
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
        Self::Split(DockItemStack {
            axis,
            items,
            sizes,
            view: stack_panel,
        })
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
        for item in items.into_iter() {
            let item: Arc<dyn PanelView> = Arc::new(item);
            new_items.push(item)
        }
        Self::new_tabs(new_items, active_ix, dock_area, cx)
    }

    fn new_tabs(
        items: Vec<Arc<dyn PanelView>>,
        active_ix: Option<usize>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        let mut new_items: Vec<Arc<dyn PanelView>> = vec![];
        let active_ix = active_ix.unwrap_or(0);
        let tab_panel = cx.new_view(|cx| {
            let mut tab_panel = TabPanel::new(None, dock_area.downgrade(), cx);

            for item in items.into_iter() {
                new_items.push(item.clone());
                tab_panel.add_panel(item.clone(), cx)
            }

            tab_panel
        });

        Self::Tabs(DockItemTabs {
            items: new_items,
            active_ix,
            view: tab_panel,
        })
    }

    pub fn tab<P: Panel>(
        item: View<P>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new_tab(Arc::new(item), dock_area, cx)
    }

    fn new_tab(
        item: Arc<dyn PanelView>,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new_tabs(vec![item], None, dock_area, cx)
    }

    /// Returns the views of the dock item.
    fn view(&self) -> Arc<dyn PanelView> {
        match self {
            Self::Split(stack) => Arc::new(stack.view.clone()),
            Self::Tabs(tabs) => Arc::new(tabs.view.clone()),
        }
    }

    /// Find existing panel in the dock item.
    pub fn find_panel(&self, panel: Arc<dyn PanelView>) -> Option<Arc<dyn PanelView>> {
        match self {
            Self::Split(stack) => stack
                .items
                .iter()
                .find_map(|item| item.find_panel(panel.clone())),
            Self::Tabs(tabs) => tabs.items.iter().find(|item| *item == &panel).cloned(),
        }
    }

    /// Find existing dock item in the dock item by panel.
    pub fn find_dock_item(&mut self, panel: Arc<dyn PanelView>) -> Option<&mut DockItem> {
        match self {
            Self::Tabs(tabs) => {
                if tabs.view.view() == panel.view() {
                    Some(self)
                } else {
                    None
                }
            }
            Self::Split(stack) => stack
                .items
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
            Self::Split(stack) => {
                // FIXME: The placement is always right, we need to add a way to specify it.
                stack.add_item(panel, Placement::Right, None, ix, &dock_area, cx);
            }
            Self::Tabs(tabs) => {
                tabs.add_item(panel, ix, cx);
            }
        }
    }

    pub(super) fn split_to(
        &mut self,
        panel: Arc<dyn PanelView>,
        parent_item: &mut DockItem,
        placement: Placement,
        dock_area: &View<DockArea>,
        cx: &mut WindowContext,
    ) {
        let new_tab = Self::new_tab(panel.clone(), dock_area, cx);

        match parent_item {
            Self::Split(DockItemStack {
                axis, items, view, ..
            }) => {
                let stack_panel = view.clone();
                let ix = items
                    .iter()
                    .position(|item| item.view() == self.view())
                    .unwrap_or_default();

                if axis.is_vertical() && placement.is_vertical() {
                    stack_panel.update(cx, |view, cx| {
                        view.insert_panel_at(
                            new_tab.view(),
                            ix,
                            placement,
                            None,
                            dock_area.downgrade(),
                            cx,
                        )
                    })
                }
            }
            _ => {
                unreachable!("Parent item must be a split")
            }
        };
    }

    /// Remove a panel from the dock item.
    pub fn remove(&mut self, panel: Arc<dyn PanelView>, cx: &mut WindowContext) {
        match self {
            Self::Split(stack) => stack.remove_item(panel, cx),
            Self::Tabs(tabs) => tabs.remove_item(panel, cx),
        }
    }
}

impl DockArea {
    pub fn new(id: impl Into<SharedString>, cx: &mut WindowContext) -> Self {
        let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Horizontal, cx));
        let dock_item = DockItem::Split(DockItemStack {
            axis: Axis::Horizontal,
            items: vec![],
            sizes: vec![],
            view: stack_panel.clone(),
        });

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
        tab_panel: View<TabPanel>,
        placement: Placement,
        size: Option<Pixels>,
        cx: &mut ViewContext<Self>,
    ) {
        let dock_area = cx.view().clone();

        // wrap the panel in a TabPanel
        let new_tab = DockItem::new_tab(panel.clone(), &dock_area, cx);

        let stack_panel = tab_panel.read(cx).parent().unwrap();
        let parent_item = self
            .items
            .find_dock_item(Arc::new(stack_panel.clone()))
            .unwrap();

        match parent_item {
            DockItem::Split(stack) => {
                stack.split_item(
                    new_tab.view(),
                    Arc::new(tab_panel),
                    placement,
                    size,
                    &dock_area,
                    cx,
                );
            }
            _ => {
                unreachable!("Parent item must be a split")
            }
        };
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
            DockItem::Split(DockItemStack { items, .. }) => {
                for item in items {
                    self.subscribe_item(item, cx);
                }
            }
            DockItem::Tabs(DockItemTabs { view, .. }) => {
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
            DockItem::Split(DockItemStack { view, .. }) => view.clone().into_any_element(),
            DockItem::Tabs(DockItemTabs { view, .. }) => view.clone().into_any_element(),
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
