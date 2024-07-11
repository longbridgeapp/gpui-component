use gpui::{
    div, px, Axis, Element, IntoElement, ParentElement as _, Render, Styled, View, ViewContext,
    VisualContext, WindowContext,
};
use ui::{
    resizable::{h_resizable, resizable_panel, ResizablePanelGroup},
    v_flex,
};

pub struct ResizableStory {
    group1: View<ResizablePanelGroup>,
}

impl ResizableStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    fn new(cx: &mut WindowContext) -> Self {
        let group1 = cx.new_view(|cx| {
            h_resizable()
                .panel(
                    resizable_panel()
                        .size(px(300.))
                        .min_size(px(100.))
                        .content(|_| div().p_4().child("Left").into_any_element()),
                    cx,
                )
                .panel(
                    resizable_panel()
                        .size(px(400.))
                        .max_size(px(550.))
                        .min_size(px(100.))
                        .content(|_| div().p_4().child("Center").into_any_element()),
                    cx,
                )
                .panel(
                    resizable_panel()
                        .size(px(300.))
                        .min_size(px(100.))
                        .content(|_| div().p_4().child("Right").into_any_element()),
                    cx,
                )
        });
        Self { group1 }
    }
}

impl Render for ResizableStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex().size_full().child(self.group1.clone())
    }
}
