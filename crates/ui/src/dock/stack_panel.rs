use std::{sync::Arc, thread::panicking};

use crate::{
    h_flex,
    resizable::{h_resizable, resizable_panel, v_resizable, ResizablePanel, ResizablePanelGroup},
    theme::ActiveTheme,
    v_flex, Placement, StyledExt,
};

use super::{Panel, PanelView, TabPanel};
use gpui::{
    div, prelude::FluentBuilder as _, px, Axis, Element, Entity, FocusHandle, FocusableView,
    IntoElement, ParentElement, Pixels, Render, Styled, View, ViewContext, VisualContext, WeakView,
};
use smallvec::SmallVec;

pub struct StackPanel {
    parent: Option<WeakView<StackPanel>>,
    pub(super) axis: Axis,
    focus_handle: FocusHandle,
    pub(super) panels: SmallVec<[Arc<dyn PanelView>; 2]>,
    panel_group: View<ResizablePanelGroup>,
}

impl Panel for StackPanel {
    fn set_size(&mut self, size: Pixels, cx: &mut gpui::WindowContext) {}
}

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

    fn insert_panel<P>(
        &mut self,
        panel: View<P>,
        ix: usize,
        size: Option<Pixels>,
        cx: &mut ViewContext<Self>,
    ) where
        P: Panel,
    {
        let view = cx.view().clone();

        // Uf the panel is a TabPanel, set its parent to this.
        if let Ok(tab_panel) = panel.view().downcast::<TabPanel>() {
            tab_panel.update(cx, |tab_panel, _| tab_panel.set_parent(view.clone()));
        }

        // If the panel is a StackPanel, set its parent to this.
        if let Ok(stack_panel) = panel.view().downcast::<StackPanel>() {
            stack_panel.update(cx, move |stack_panel, _| {
                stack_panel.parent = Some(view.downgrade());
            });
        }

        self.panel_group.update(cx, |view, cx| {
            let size_panel = resizable_panel()
                .content_view(panel.view())
                .min_size(px(100.))
                .when_some(size, |this, size| this.size(size))
                .when(size.is_none(), |this| this.grow());

            view.insert_child(size_panel, ix, cx)
        });

        cx.notify();
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

    pub fn remove_panel<P>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        if let Some(ix) = self.index_of_panel(panel) {
            self.panels.remove(ix);
            self.panel_group.update(cx, |view, cx| {
                view.remove_child(ix, cx);
            });

            // If children is empty, remove self from parent view.
            if self.panels.is_empty() {
                let view = cx.view().clone();
                if let Some(parent) = self.parent.as_ref().and_then(|p| p.upgrade()) {
                    parent.update(cx, |parent, cx| {
                        parent.remove_panel(view, cx);
                    });
                }

                cx.notify();
            }
        }
    }
}

impl FocusableView for StackPanel {
    fn focus_handle(&self, cx: &gpui::AppContext) -> FocusHandle {
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
