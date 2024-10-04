use std::sync::Arc;

use crate::{
    dock::DockItemInfo,
    h_flex,
    resizable::{
        h_resizable, resizable_panel, v_resizable, ResizablePanel, ResizablePanelEvent,
        ResizablePanelGroup,
    },
    theme::ActiveTheme,
    AxisExt, Placement,
};

use super::{DockArea, DockItemState, Panel, PanelEvent, PanelView, TabPanel};
use gpui::{
    prelude::FluentBuilder as _, AppContext, Axis, DismissEvent, Entity, EventEmitter, FocusHandle,
    FocusableView, IntoElement, ParentElement, Pixels, Render, Styled, View, ViewContext,
    VisualContext, WeakView,
};
use smallvec::SmallVec;

pub struct StackPanel {
    pub(super) parent: Option<View<StackPanel>>,
    pub(super) axis: Axis,
    focus_handle: FocusHandle,
    pub(crate) panels: SmallVec<[Arc<dyn PanelView>; 2]>,
    panel_group: View<ResizablePanelGroup>,
}

impl Panel for StackPanel {
    fn panel_name(&self) -> &'static str {
        "StackPanel"
    }

    fn title(&self, _cx: &gpui::WindowContext) -> gpui::AnyElement {
        "StackPanel".into_any_element()
    }

    fn dump(&self, cx: &AppContext) -> DockItemState {
        let sizes = self.panel_group.read(cx).sizes();
        let mut state = DockItemState::new(self.panel_name());
        for panel in &self.panels {
            state.add_child(panel.dump(cx));
            state.info = DockItemInfo::stack(sizes.clone(), self.axis);
        }

        state
    }
}

impl StackPanel {
    pub fn new(axis: Axis, cx: &mut ViewContext<Self>) -> Self {
        let panel_group = cx.new_view(|cx| {
            if axis == Axis::Horizontal {
                h_resizable(cx)
            } else {
                v_resizable(cx)
            }
        });

        // Bubble up the resize event.
        cx.subscribe(&panel_group, |_, _, _: &ResizablePanelEvent, cx| {
            cx.emit(PanelEvent::LayoutChanged)
        })
        .detach();

        Self {
            axis,
            parent: None,
            focus_handle: cx.focus_handle(),
            panels: SmallVec::new(),
            panel_group,
        }
    }

    /// The first level of the stack panel is root, will not have a parent.
    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub(super) fn panels_len(&self) -> usize {
        self.panels.len()
    }

    /// Return the index of the panel.
    pub(crate) fn index_of_panel(&self, panel: Arc<dyn PanelView>) -> Option<usize> {
        self.panels.iter().position(|p| p == &panel)
    }

    /// Add a panel at the end of the stack.
    pub fn add_panel(
        &mut self,
        panel: Arc<dyn PanelView>,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel(panel, self.panels.len(), size, dock_area, cx);
    }

    pub fn add_panel_at(
        &mut self,
        panel: Arc<dyn PanelView>,
        placement: Placement,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel_at(panel, self.panels_len(), placement, size, dock_area, cx);
    }

    pub fn insert_panel_at(
        &mut self,
        panel: Arc<dyn PanelView>,
        ix: usize,
        placement: Placement,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) {
        match placement {
            Placement::Top | Placement::Left => {
                self.insert_panel_before(panel, ix, size, dock_area, cx)
            }
            Placement::Right | Placement::Bottom => {
                self.insert_panel_after(panel, ix, size, dock_area, cx)
            }
        }
    }

    /// Insert a panel at the index.
    pub fn insert_panel_before(
        &mut self,
        panel: Arc<dyn PanelView>,
        ix: usize,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel(panel, ix, size, dock_area, cx);
    }

    /// Insert a panel after the index.
    pub fn insert_panel_after(
        &mut self,
        panel: Arc<dyn PanelView>,
        ix: usize,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) {
        self.insert_panel(panel, ix + 1, size, dock_area, cx);
    }

    fn new_resizable_panel(panel: Arc<dyn PanelView>, size: Option<Pixels>) -> ResizablePanel {
        resizable_panel()
            .content_view(panel.view())
            .when_some(size, |this, size| this.size(size))
    }

    fn insert_panel(
        &mut self,
        panel: Arc<dyn PanelView>,
        ix: usize,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) {
        // If the panel is already in the stack, return.
        if let Some(_) = self.index_of_panel(panel.clone()) {
            return;
        }

        let view = cx.view().clone();
        cx.window_context().defer({
            let panel = panel.clone();

            move |cx| {
                // If the panel is a TabPanel, set its parent to this.
                if let Ok(tab_panel) = panel.view().downcast::<TabPanel>() {
                    tab_panel.update(cx, |tab_panel, _| tab_panel.set_parent(view));
                } else if let Ok(stack_panel) = panel.view().downcast::<Self>() {
                    stack_panel.update(cx, |stack_panel, _| stack_panel.parent = Some(view));
                }

                // Subscribe to the panel's layout change event.
                _ = dock_area.update(cx, |_, cx| {
                    if let Ok(tab_panel) = panel.view().downcast::<TabPanel>() {
                        DockArea::subscribe_panel(&tab_panel, cx);
                    }
                });
            }
        });

        let ix = if ix > self.panels.len() {
            self.panels.len()
        } else {
            ix
        };

        self.panels.insert(ix, panel.clone());
        self.panel_group.update(cx, |view, cx| {
            view.insert_child(Self::new_resizable_panel(panel.clone(), size), ix, cx)
        });

        cx.emit(PanelEvent::LayoutChanged);
        cx.notify();
    }

    /// Remove panel from the stack.
    pub fn remove_panel(&mut self, panel: Arc<dyn PanelView>, cx: &mut ViewContext<Self>) {
        if let Some(ix) = self.index_of_panel(panel.clone()) {
            self.panels.remove(ix);
            self.panel_group.update(cx, |view, cx| {
                view.remove_child(ix, cx);
            });

            cx.emit(PanelEvent::LayoutChanged);
            self.remove_self_if_empty(cx);
        } else {
            println!("Panel not found in stack panel.");
        }
    }

    /// Replace the old panel with the new panel at same index.
    pub(super) fn replace_panel(
        &mut self,
        old_panel: Arc<dyn PanelView>,
        new_panel: View<StackPanel>,
        cx: &mut ViewContext<Self>,
    ) {
        if let Some(ix) = self.index_of_panel(old_panel.clone()) {
            self.panels[ix] = Arc::new(new_panel.clone());
            self.panel_group.update(cx, |view, cx| {
                view.replace_child(
                    Self::new_resizable_panel(Arc::new(new_panel.clone()), None),
                    ix,
                    cx,
                );
            });
            cx.emit(PanelEvent::LayoutChanged);
        }
    }

    /// If children is empty, remove self from parent view.
    pub(crate) fn remove_self_if_empty(&mut self, cx: &mut ViewContext<Self>) {
        if self.is_root() {
            return;
        }

        if !self.panels.is_empty() {
            return;
        }

        let view = cx.view().clone();
        if let Some(parent) = self.parent.as_ref() {
            parent.update(cx, |parent, cx| {
                parent.remove_panel(Arc::new(view.clone()), cx);
            });
        }

        cx.emit(PanelEvent::LayoutChanged);
        cx.notify();
    }

    /// Remove all panels from the stack.
    pub(super) fn remove_all_panels(&mut self, cx: &mut ViewContext<Self>) {
        self.panels.clear();
        self.panel_group
            .update(cx, |view, cx| view.remove_all_children(cx));
    }

    /// Change the axis of the stack panel.
    pub(super) fn set_axis(&mut self, axis: Axis, cx: &mut ViewContext<Self>) {
        self.axis = axis;
        self.panel_group
            .update(cx, |view, cx| view.set_axis(axis, cx));
        cx.notify();
    }

    /// Check if the given panel is at the first top left in the stack.
    pub(super) fn is_top_left_panel(
        &self,
        panel: View<TabPanel>,
        check_parent: bool,
        cx: &AppContext,
    ) -> bool {
        let first_panel = self.panels.first();

        if let Some(parent) = &self.parent {
            if check_parent {
                return parent.read(cx).is_top_left_panel(panel, true, cx);
            }
        }

        if let Some(view) = first_panel {
            if let Ok(view) = view.view().downcast::<TabPanel>() {
                return view.entity_id() == panel.entity_id();
            } else if let Ok(view) = view.view().downcast::<Self>() {
                return view.read(cx).is_top_left_panel(panel, false, cx);
            }
        }
        false
    }

    /// Check if the given panel is at the first top right in the stack.
    pub(super) fn is_top_right_panel(
        &self,
        panel: View<TabPanel>,
        check_parent: bool,
        cx: &AppContext,
    ) -> bool {
        let first_panel = if self.axis.is_vertical() {
            self.panels.first()
        } else {
            self.panels.last()
        };

        if let Some(parent) = &self.parent {
            if check_parent {
                return parent.read(cx).is_top_right_panel(panel, true, cx);
            }
        }

        if let Some(view) = first_panel {
            if let Ok(view) = view.view().downcast::<TabPanel>() {
                return view.entity_id() == panel.entity_id();
            } else if let Ok(view) = view.view().downcast::<Self>() {
                return view.read(cx).is_top_right_panel(panel, false, cx);
            }
        }

        false
    }
}

impl FocusableView for StackPanel {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl EventEmitter<PanelEvent> for StackPanel {}
impl EventEmitter<DismissEvent> for StackPanel {}
impl Render for StackPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().tab_bar)
            .child(self.panel_group.clone())
    }
}
