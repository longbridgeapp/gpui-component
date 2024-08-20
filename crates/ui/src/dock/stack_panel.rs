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
    IntoElement, ParentElement, Pixels, Render, Styled, View, ViewContext, VisualContext,
};
use smallvec::SmallVec;

pub struct StackPanel {
    axis: Axis,
    focus_handle: FocusHandle,
    children: SmallVec<[Arc<dyn PanelView>; 2]>,
    panel_group: View<ResizablePanelGroup>,
    placement: Placement,
}

impl Panel for StackPanel {
    fn set_size(&mut self, size: Pixels, cx: &mut gpui::WindowContext) {}

    fn set_placement(&mut self, placement: Placement, cx: &mut gpui::WindowContext) {
        self.placement = placement;
    }

    fn placement(&self, cx: &gpui::WindowContext) -> Placement {
        self.placement
    }
}

impl StackPanel {
    pub fn new(axis: Axis, cx: &mut ViewContext<Self>) -> Self {
        Self {
            axis,
            focus_handle: cx.focus_handle(),
            children: SmallVec::new(),
            panel_group: cx.new_view(|_| {
                if axis == Axis::Horizontal {
                    h_resizable()
                } else {
                    v_resizable()
                }
            }),
            placement: Placement::Left,
        }
    }

    /// Add a panel at the end of the stack.
    pub fn add_panel<P>(&mut self, panel: View<P>, size: Option<Pixels>, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        let view = cx.view().clone();
        if let Ok(tab_panel) = panel.view().downcast::<TabPanel>() {
            tab_panel.update(cx, |tab_panel, _| tab_panel.set_parent(view));
        }

        self.panel_group.update(cx, |view, cx| {
            let size_panel = resizable_panel()
                .content_view(panel.view())
                .min_size(px(100.))
                .when_some(size, |this, size| this.size(size))
                .when(size.is_none(), |this| this.grow());

            view.add_child(size_panel, cx)
        });
        // self.children.push(Arc::new(panel));
    }

    /// Insert a panel at the index.
    pub fn insert_panel_before<P>(&mut self, panel: View<P>, ix: usize)
    where
        P: Panel,
    {
        self.children.insert(ix, Arc::new(panel));
    }

    /// Insert a panel after the index.
    pub fn insert_panel_after<P>(&mut self, panel: View<P>, ix: usize)
    where
        P: Panel,
    {
        self.children.insert(ix + 1, Arc::new(panel));
    }

    pub fn remove_panel<P>(&mut self, panel: View<P>)
    where
        P: Panel,
    {
        let entity_id = panel.entity_id();
        self.children.retain(|p| p.view().entity_id() != entity_id);
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
