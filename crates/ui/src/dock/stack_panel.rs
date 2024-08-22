use std::sync::Arc;

use crate::{
    resizable::{h_resizable, resizable_panel, v_resizable, ResizablePanel, ResizablePanelGroup},
    theme::ActiveTheme,
    Placement,
};

use super::{Panel, PanelView, TabPanel};
use gpui::{
    div, prelude::FluentBuilder as _, px, Axis, Entity, FocusHandle, FocusableView, IntoElement,
    ParentElement, Pixels, Render, Styled, View, ViewContext, VisualContext,
};
use smallvec::SmallVec;

pub struct StackPanel {
    parent: Option<View<StackPanel>>,
    pub(super) axis: Axis,
    focus_handle: FocusHandle,
    pub(super) panels: SmallVec<[Arc<dyn PanelView>; 2]>,
    panel_group: View<ResizablePanelGroup>,
}

impl Panel for StackPanel {}

impl StackPanel {
    pub fn new(axis: Axis, cx: &mut ViewContext<Self>) -> Self {
        Self {
            axis,
            parent: None,
            focus_handle: cx.focus_handle(),
            panels: SmallVec::new(),
            panel_group: cx.new_view(|_| {
                if axis == Axis::Horizontal {
                    h_resizable()
                } else {
                    v_resizable()
                }
            }),
        }
    }

    /// The first level of the stack panel is root, will not have a parent.
    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Add a panel at the end of the stack.
    pub fn add_panel<P>(&mut self, panel: View<P>, size: Option<Pixels>, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        self.insert_panel(panel, self.panels.len(), size, cx);
    }

    /// Return the index of the panel.
    pub fn index_of_panel<P>(&self, panel: View<P>) -> Option<usize>
    where
        P: Panel,
    {
        let entity_id = panel.entity_id();
        self.panels
            .iter()
            .position(|p| p.view().entity_id() == entity_id)
    }

    pub fn add_panel_at<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        placement: Placement,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        match placement {
            Placement::Top | Placement::Left => self.insert_panel_before(panel, ix, cx),
            Placement::Right | Placement::Bottom => self.insert_panel_after(panel, ix, cx),
        }
    }

    /// Insert a panel at the index.
    pub fn insert_panel_before<P>(&mut self, panel: View<P>, ix: usize, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        self.insert_panel(panel, ix, None, cx);
    }

    /// Insert a panel after the index.
    pub fn insert_panel_after<P>(&mut self, panel: View<P>, ix: usize, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        self.insert_panel(panel, ix + 1, None, cx);
    }

    fn new_resizable_panel<P>(panel: View<P>, size: Option<Pixels>) -> ResizablePanel
    where
        P: Panel,
    {
        resizable_panel()
            .content_view(panel.view())
            .min_size(px(100.))
            .when_some(size, |this, size| this.size(size))
            .when(size.is_none(), |this| this.grow())
    }

    fn insert_panel<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        size: Option<Pixels>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        // If the panel is already in the stack, return.
        if let Some(_) = self.index_of_panel(panel.clone()) {
            return;
        }

        cx.spawn(|view, mut cx| {
            let panel = panel.clone();
            async move {
                if let Some(view) = view.upgrade() {
                    cx.update(|cx| {
                        // If the panel is a TabPanel, set its parent to this.
                        if let Ok(tab_panel) = panel.view().downcast::<TabPanel>() {
                            tab_panel.update(cx, |tab_panel, _| tab_panel.set_parent(view.clone()));
                        } else if let Ok(stack_panel) = panel.view().downcast::<Self>() {
                            stack_panel.update(cx, |stack_panel, _| {
                                stack_panel.parent = Some(view.clone())
                            });
                        }
                    })
                } else {
                    Ok(())
                }
            }
        })
        .detach();

        self.panels.insert(ix, Arc::new(panel.clone()));
        self.panel_group.update(cx, |view, cx| {
            view.insert_child(Self::new_resizable_panel(panel, size), ix, cx)
        });

        cx.notify();
    }

    pub fn replace_panel<P, P1>(
        &mut self,
        old_panel: View<P>,
        new_panel: View<P1>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
        P1: Panel,
    {
        if let Some(ix) = self.index_of_panel(old_panel) {
            self.panels[ix] = Arc::new(new_panel.clone());
            self.panel_group.update(cx, |view, cx| {
                view.replace_child(Self::new_resizable_panel(new_panel, None), ix, cx);
            });
        }
    }

    /// Remove panel from the stack.
    pub fn remove_panel<P>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        if let Some(ix) = self.index_of_panel(panel) {
            self.panels.remove(ix);
            self.panel_group.update(cx, |view, cx| {
                view.remove_child(ix, cx);
            });

            self.remove_self_if_empty(cx);
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
}

impl FocusableView for StackPanel {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for StackPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().tab_bar)
            .child(self.panel_group.clone())
    }
}
