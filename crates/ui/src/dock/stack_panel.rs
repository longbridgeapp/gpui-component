use std::sync::Arc;

use crate::{
    h_flex,
    resizable::{h_resizable, resizable_panel, v_resizable, ResizablePanel, ResizablePanelGroup},
    theme::ActiveTheme,
    v_flex, Placement, StyledExt,
};

use super::{Panel, PanelView};
use gpui::{
    div, Axis, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext,
};
use smallvec::SmallVec;

pub struct StackPanel {
    axis: Axis,
    children: SmallVec<[Arc<dyn PanelView>; 2]>,
    panel_group: View<ResizablePanelGroup>,
    placement: Placement,
}

impl StackPanel {
    pub fn new(axis: Axis, cx: &mut ViewContext<Self>) -> Self {
        Self {
            axis,
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
    pub fn add_panel<P>(&mut self, panel: View<P>, cx: &mut ViewContext<Self>)
    where
        P: Panel,
    {
        self.panel_group.update(cx, |view, cx| {
            view.add_child(resizable_panel().content_view(panel.into_any()), cx)
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
}

impl Render for StackPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex_1()
            .overflow_hidden()
            .bg(cx.theme().tab_bar)
            .child(self.panel_group.clone())
    }
}
