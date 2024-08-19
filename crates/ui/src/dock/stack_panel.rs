use std::sync::Arc;

use crate::{h_flex, v_flex, Placement, StyledExt};

use super::{Panel, PanelView};
use gpui::{
    Axis, IntoElement, ParentElement, Pixels, Render, SharedString, View, ViewContext,
    WindowContext,
};
use smallvec::SmallVec;

pub struct StackPanel {
    axis: Axis,
    children: SmallVec<[Arc<dyn PanelView>; 2]>,
    placement: Placement,
}

impl StackPanel {
    pub fn new(axis: Axis, cx: &ViewContext<Self>) -> Self {
        Self {
            axis,
            children: SmallVec::new(),
            placement: Placement::Left,
        }
    }

    /// Add a panel at the end of the stack.
    pub fn add_panel<P>(&mut self, panel: View<P>)
    where
        P: Panel,
    {
        self.children.push(Arc::new(panel));
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
        match self.axis {
            Axis::Horizontal => h_flex(),
            Axis::Vertical => v_flex(),
        }
        .debug_red()
        .children(self.children.clone().into_iter().map(|c| c.into_any()))
    }
}
