mod dock;
mod invalid_panel;
mod panel;
mod stack_panel;
mod state;
mod tab_panel;
mod toggle_buttons;

use anyhow::Result;
pub use dock::*;
use gpui::{
    actions, canvas, div, prelude::FluentBuilder, AnyElement, AnyView, AppContext, Axis, Bounds,
    Entity as _, EntityId, EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Render, SharedString, Styled, Subscription, View, ViewContext, VisualContext, WeakView,
    WindowContext,
};
use std::sync::Arc;

pub use panel::*;
pub use stack_panel::*;
pub use state::*;
pub use tab_panel::*;
pub use toggle_buttons::*;

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
    /// The version is used to special the default layout, this is like the `panel_version` in `trait Panel`.
    version: Option<usize>,
    pub(crate) bounds: Bounds<Pixels>,

    /// The center view of the dockarea.
    items: DockItem,

    /// The entity_id of the TabPanel where each toggle button should be displayed,
    left_toggle_button_tab_panel_id: Option<EntityId>,
    right_toggle_button_tab_panel_id: Option<EntityId>,
    bottom_toggle_button_tab_panel_id: Option<EntityId>,

    /// The left dock of the dockarea.
    left_dock: Option<View<Dock>>,
    /// The bottom dock of the dockarea.
    bottom_dock: Option<View<Dock>>,
    /// The right dock of the dockarea.
    right_dock: Option<View<Dock>>,
    /// The top zoom view of the dockarea, if any.
    zoom_view: Option<AnyView>,

    /// Lock panels layout, but allow to resize.
    is_locked: bool,

    _subscriptions: Vec<Subscription>,
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

        cx.defer({
            let stack_panel = stack_panel.clone();
            let dock_area = dock_area.clone();
            move |cx| {
                _ = dock_area.update(cx, |this, cx| {
                    this.subscribe_panel(&stack_panel, cx);
                });
            }
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

    pub fn set_collapsed(&self, collapsed: bool, cx: &mut WindowContext) {
        match self {
            DockItem::Tabs { view, .. } => {
                view.update(cx, |tab_panel, cx| {
                    tab_panel.set_collapsed(collapsed, cx);
                });
            }
            DockItem::Split { items, .. } => {
                // For each child item, set collapsed state
                for item in items {
                    item.set_collapsed(collapsed, cx);
                }
            }
        }
    }

    /// Recursively checks if the DockItem or any of its children contain the entity_id of the TabPanel
    pub fn contains_entity_id(&self, entity_id: EntityId) -> bool {
        match self {
            DockItem::Tabs { view, .. } => view.entity_id() == entity_id,
            DockItem::Split { items, .. } => {
                items.iter().any(|item| item.contains_entity_id(entity_id))
            }
        }
    }

    /// Recursively traverses to find the left-most and top-most TabPanel.
    pub fn left_top_tab_panel(&self, cx: &AppContext) -> Option<View<TabPanel>> {
        match self {
            DockItem::Tabs { view, .. } => Some(view.clone()),
            DockItem::Split { view, .. } => view.read(cx).left_top_tab_panel(true, cx),
        }
    }

    /// Recursively traverses to find the right-most and top-most TabPanel.
    pub fn right_top_tab_panel(&self, cx: &AppContext) -> Option<View<TabPanel>> {
        match self {
            DockItem::Tabs { view, .. } => Some(view.clone()),
            DockItem::Split { view, .. } => view.read(cx).right_top_tab_panel(true, cx),
        }
    }
}

impl DockArea {
    pub fn new(
        id: impl Into<SharedString>,
        version: Option<usize>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let stack_panel = cx.new_view(|cx| StackPanel::new(Axis::Horizontal, cx));

        let dock_item = DockItem::Split {
            axis: Axis::Horizontal,
            items: vec![],
            sizes: vec![],
            view: stack_panel.clone(),
        };

        let mut this = Self {
            id: id.into(),
            version,
            bounds: Bounds::default(),
            items: dock_item,
            zoom_view: None,
            left_toggle_button_tab_panel_id: None,
            right_toggle_button_tab_panel_id: None,
            bottom_toggle_button_tab_panel_id: None,
            left_dock: None,
            right_dock: None,
            bottom_dock: None,
            is_locked: false,
            _subscriptions: vec![],
        };

        this.subscribe_panel(&stack_panel, cx);

        this
    }

    /// Set version of the dock area.
    pub fn set_version(&mut self, version: usize, cx: &mut ViewContext<Self>) {
        self.version = Some(version);
        cx.notify();
    }

    /// The the DockItem as the root of the dock area.
    ///
    /// This is used to render at the Center of the DockArea.
    pub fn set_root(&mut self, item: DockItem, cx: &mut ViewContext<Self>) {
        self.subscribe_item(&item, cx);
        self.items = item;

        cx.notify();
    }

    pub fn set_left_dock(
        &mut self,
        panel: DockItem,
        size: Option<Pixels>,
        cx: &mut ViewContext<Self>,
    ) {
        self.subscribe_item(&panel, cx);
        let weak_self = cx.view().downgrade();
        self.left_dock = Some(cx.new_view(|cx| {
            let mut dock = Dock::left(weak_self.clone(), cx);
            if let Some(size) = size {
                dock.set_size(size, cx);
            }
            dock.set_panel(panel, cx);
            dock
        }));
    }

    pub fn set_bottom_dock(
        &mut self,
        panel: DockItem,
        size: Option<Pixels>,
        cx: &mut ViewContext<Self>,
    ) {
        self.subscribe_item(&panel, cx);
        let weak_self = cx.view().downgrade();
        self.bottom_dock = Some(cx.new_view(|cx| {
            let mut dock = Dock::bottom(weak_self.clone(), cx);
            if let Some(size) = size {
                dock.set_size(size, cx);
            }
            dock.set_panel(panel, cx);
            dock
        }));
    }

    pub fn set_right_dock(
        &mut self,
        panel: DockItem,
        size: Option<Pixels>,
        cx: &mut ViewContext<Self>,
    ) {
        self.subscribe_item(&panel, cx);
        let weak_self = cx.view().downgrade();
        self.right_dock = Some(cx.new_view(|cx| {
            let mut dock = Dock::right(weak_self.clone(), cx);
            if let Some(size) = size {
                dock.set_size(size, cx);
            }
            dock.set_panel(panel, cx);
            dock
        }));
    }

    /// Set locked state of the dock area, if locked, the dock area cannot be split or move, but allows to resize panels.
    pub fn set_locked(&mut self, locked: bool, _: &mut WindowContext) {
        self.is_locked = locked;
    }

    /// Determine if the dock area is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Determine if the dock area has a dock at the given placement.
    pub fn has_dock(&self, placement: DockPlacement) -> bool {
        match placement {
            DockPlacement::Left => self.left_dock.is_some(),
            DockPlacement::Bottom => self.bottom_dock.is_some(),
            DockPlacement::Right => self.right_dock.is_some(),
        }
    }

    /// Determine if the dock at the given placement is open.
    pub fn is_dock_open(&self, placement: DockPlacement, cx: &AppContext) -> bool {
        match placement {
            DockPlacement::Left => self
                .left_dock
                .as_ref()
                .map(|dock| dock.read(cx).is_open())
                .unwrap_or(false),
            DockPlacement::Bottom => self
                .bottom_dock
                .as_ref()
                .map(|dock| dock.read(cx).is_open())
                .unwrap_or(false),
            DockPlacement::Right => self
                .right_dock
                .as_ref()
                .map(|dock| dock.read(cx).is_open())
                .unwrap_or(false),
        }
    }

    pub fn toggle_dock(&self, placement: DockPlacement, cx: &mut ViewContext<Self>) {
        let dock = match placement {
            DockPlacement::Left => &self.left_dock,
            DockPlacement::Bottom => &self.bottom_dock,
            DockPlacement::Right => &self.right_dock,
        };
        if let Some(dock) = dock {
            dock.update(cx, |view, cx| {
                view.toggle_open(cx);
            })
        }
    }

    /// Load the state of the DockArea from the DockAreaState.
    ///
    /// See also [DockeArea::dump].
    pub fn load(&mut self, state: DockAreaState, cx: &mut ViewContext<Self>) -> Result<()> {
        self.version = state.version;
        let weak_self = cx.view().downgrade();

        if let Some(left_dock_state) = state.left_dock {
            self.left_dock = Some(left_dock_state.to_dock(weak_self.clone(), cx));
        }

        if let Some(right_dock_state) = state.right_dock {
            self.right_dock = Some(right_dock_state.to_dock(weak_self.clone(), cx));
        }

        if let Some(bottom_dock_state) = state.bottom_dock {
            self.bottom_dock = Some(bottom_dock_state.to_dock(weak_self.clone(), cx));
        }

        self.items = state.center.to_item(weak_self, cx);
        self.update_toggle_button_tab_panels(cx);
        Ok(())
    }

    /// Dump the dock panels layout to DockItemState.
    ///
    /// See also [DockArea::load].
    pub fn dump(&self, cx: &AppContext) -> DockAreaState {
        let root = self.items.view();
        let center = root.dump(cx);

        let left_dock = self
            .left_dock
            .as_ref()
            .map(|dock| DockState::new(dock.clone(), cx));
        let right_dock = self
            .right_dock
            .as_ref()
            .map(|dock| DockState::new(dock.clone(), cx));
        let bottom_dock = self
            .bottom_dock
            .as_ref()
            .map(|dock| DockState::new(dock.clone(), cx));

        DockAreaState {
            version: self.version,
            center,
            left_dock,
            right_dock,
            bottom_dock,
        }
    }

    /// Subscribe event on the panels
    #[allow(clippy::only_used_in_recursion)]
    fn subscribe_item(&mut self, item: &DockItem, cx: &mut ViewContext<Self>) {
        match item {
            DockItem::Split { items, view, .. } => {
                for item in items {
                    self.subscribe_item(item, cx);
                }

                self._subscriptions
                    .push(cx.subscribe(view, move |_, _, event, cx| match event {
                        PanelEvent::LayoutChanged => {
                            let dock_area = cx.view().clone();
                            cx.spawn(|_, mut cx| async move {
                                let _ = cx.update(|cx| {
                                    let _ = dock_area.update(cx, |view, cx| {
                                        view.update_toggle_button_tab_panels(cx)
                                    });
                                });
                            })
                            .detach();
                            cx.emit(DockEvent::LayoutChanged);
                        }
                        _ => {}
                    }));
            }
            DockItem::Tabs { .. } => {
                // We subscribe to the tab panel event in StackPanel's insert_panel
            }
        }
    }

    /// Subscribe zoom event on the panel
    pub(crate) fn subscribe_panel<P: Panel>(
        &mut self,
        view: &View<P>,
        cx: &mut ViewContext<DockArea>,
    ) {
        let subscription = cx.subscribe(view, move |_, panel, event, cx| match event {
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
            PanelEvent::LayoutChanged => {
                let dock_area = cx.view().clone();
                cx.spawn(|_, mut cx| async move {
                    let _ = cx.update(|cx| {
                        let _ = dock_area
                            .update(cx, |view, cx| view.update_toggle_button_tab_panels(cx));
                    });
                })
                .detach();
                cx.emit(DockEvent::LayoutChanged);
            }
        });

        self._subscriptions.push(subscription);
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

    pub fn update_toggle_button_tab_panels(&mut self, cx: &mut ViewContext<Self>) {
        // Left toggle button
        self.left_toggle_button_tab_panel_id = self
            .items
            .left_top_tab_panel(cx)
            .map(|view| view.entity_id());

        // Right toggle button
        self.right_toggle_button_tab_panel_id = self
            .items
            .right_top_tab_panel(cx)
            .map(|view| view.entity_id());

        // Bottom toggle button
        self.bottom_toggle_button_tab_panel_id = self
            .bottom_dock
            .as_ref()
            .and_then(|dock| dock.read(cx).panel.left_top_tab_panel(cx))
            .map(|view| view.entity_id());
    }
}
impl EventEmitter<DockEvent> for DockArea {}
impl Render for DockArea {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();

        div()
            .id("dock-area")
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
                        div()
                            .flex()
                            .flex_row()
                            .h_full()
                            // Left dock
                            .when_some(self.left_dock.clone(), |this, dock| {
                                this.child(div().flex().flex_none().child(dock))
                            })
                            // Center
                            .child(
                                div()
                                    .flex()
                                    .flex_1()
                                    .flex_col()
                                    .overflow_hidden()
                                    // Top center
                                    .child(
                                        div()
                                            .flex_1()
                                            .overflow_hidden()
                                            .child(self.render_items(cx)),
                                    )
                                    // Bottom Dock
                                    .when_some(self.bottom_dock.clone(), |this, dock| {
                                        this.child(dock)
                                    }),
                            )
                            // Right Dock
                            .when_some(self.right_dock.clone(), |this, dock| {
                                this.child(div().flex().flex_none().child(dock))
                            }),
                    )
                }
            })
    }
}
