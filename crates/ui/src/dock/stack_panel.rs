use std::sync::Arc;

use crate::{
    h_flex,
    resizable::{h_resizable, resizable_panel, v_resizable, ResizablePanel, ResizablePanelGroup},
    theme::ActiveTheme,
    Placement,
};

use super::{DockArea, Panel, PanelEvent, PanelView, TabPanel};
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
    panels: SmallVec<[Arc<dyn PanelView>; 2]>,
    panel_group: View<ResizablePanelGroup>,
}

impl Panel for StackPanel {
    fn title(&self, _cx: &gpui::WindowContext) -> gpui::SharedString {
        "StackPanel".into()
    }
}

impl StackPanel {
    pub fn new(axis: Axis, cx: &mut ViewContext<Self>) -> Self {
        Self {
            axis,
            parent: None,
            focus_handle: cx.focus_handle(),
            panels: SmallVec::new(),
            panel_group: cx.new_view(|cx| {
                if axis == Axis::Horizontal {
                    h_resizable(cx)
                } else {
                    v_resizable(cx)
                }
            }),
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
    pub(crate) fn index_of_panel<P>(&self, panel: &View<P>) -> Option<usize>
    where
        P: Panel,
    {
        let entity_id = panel.entity_id();
        self.panels
            .iter()
            .position(|p| p.view().entity_id() == entity_id)
    }

    /// Add a panel at the end of the stack.
    pub fn add_panel<P>(
        &mut self,
        panel: View<P>,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        self.insert_panel(panel, self.panels.len(), size, dock_area, cx);
    }

    pub fn add_panel_at<P>(
        &mut self,
        panel: View<P>,
        placement: Placement,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        self.insert_panel_at(panel, self.panels_len(), placement, size, dock_area, cx);
    }

    pub fn insert_panel_at<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        placement: Placement,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
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
    pub fn insert_panel_before<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        self.insert_panel(panel, ix, size, dock_area, cx);
    }

    /// Insert a panel after the index.
    pub fn insert_panel_after<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        self.insert_panel(panel, ix + 1, size, dock_area, cx);
    }

    fn new_resizable_panel<P>(panel: View<P>, size: Option<Pixels>) -> ResizablePanel
    where
        P: Panel,
    {
        resizable_panel()
            .content_view(panel.into())
            .when_some(size, |this, size| this.size(size))
    }

    fn insert_panel<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        size: Option<Pixels>,
        dock_area: WeakView<DockArea>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        // If the panel is already in the stack, return.
        if let Some(_) = self.index_of_panel(&panel) {
            return;
        }

        cx.subscribe(&panel, move |_, panel, event, cx| match event {
            PanelEvent::ZoomIn => {
                let _ = dock_area.update(cx, |dock, cx| {
                    dock.set_zoomed_in(panel.clone(), cx);
                });
            }
            PanelEvent::ZoomOut => {
                let _ = dock_area.update(cx, |dock, cx| dock.set_zoomed_out(cx));
            }
        })
        .detach();

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
            }
        });

        let ix = if ix > self.panels.len() {
            self.panels.len()
        } else {
            ix
        };

        self.panels.insert(ix, Arc::new(panel.clone()));
        self.panel_group.update(cx, |view, cx| {
            view.insert_child(Self::new_resizable_panel(panel, size), ix, cx)
        });

        cx.notify();
    }

    /// Remove panel from the stack.
    pub fn remove_panel<P>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        if let Some(ix) = self.index_of_panel(&panel) {
            self.panels.remove(ix);
            self.panel_group.update(cx, |view, cx| {
                view.remove_child(ix, cx);
            });

            self.remove_self_if_empty(cx);
        } else {
            println!("Panel not found in stack panel.");
        }
    }

    /// Replace the old panel with the new panel at same index.
    pub(super) fn replace_panel<P>(
        &mut self,
        old_panel: View<P>,
        new_panel: View<StackPanel>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        if let Some(ix) = self.index_of_panel(&old_panel) {
            self.panels[ix] = Arc::new(new_panel.clone());
            self.panel_group.update(cx, |view, cx| {
                view.replace_child(Self::new_resizable_panel(new_panel.clone(), None), ix, cx);
            });
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
                parent.remove_panel(view, cx);
            });
        }

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
