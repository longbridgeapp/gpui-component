use gpui::{
    div, px, AnyView, IntoElement, ParentElement, Render, SharedString, Styled, ViewContext,
    VisualContext, WindowContext,
};

use crate::{theme::ActiveTheme, StyledExt};

pub struct Tooltip {
    text: SharedString,
}

impl Tooltip {
    pub fn new(text: impl Into<SharedString>, cx: &mut WindowContext) -> AnyView {
        cx.new_view(|_| Self { text: text.into() }).into()
    }
}

impl Render for Tooltip {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(
            // Wrap in a child, to ensure the left margin is applied to the tooltip
            div()
                .m_3()
                .bg(cx.theme().popover)
                .text_color(cx.theme().popover_foreground)
                .elevation_1(cx)
                .rounded(px(6.))
                .pt_1()
                .pb_0p5()
                .px_2()
                .text_sm()
                .child(self.text.clone()),
        )
    }
}
