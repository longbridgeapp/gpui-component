use crate::h_flex;
use gpui::{
    prelude::FluentBuilder as _, AnyElement, IntoElement, ParentElement as _, Render, View,
    ViewContext,
};

use super::Pane;

pub struct DockArea<P: Pane> {
    panels: Vec<P>,
}

impl<P> DockArea<P>
where
    P: Pane,
{
    pub fn new() -> Self {
        Self { panels: Vec::new() }
    }
}

impl<P> Render for DockArea<P>
where
    P: Pane,
{
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .when_some(self.left.clone(), |this, panel| this.child(panel))
            .child
            .when_some(self.right.clone(), |this, panel| this.child(panel))
    }
}
