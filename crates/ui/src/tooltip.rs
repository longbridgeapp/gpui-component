use gpui::{
    div, px, AnyView, IntoElement, ParentElement, Render, SharedString, Styled, ViewContext,
    VisualContext, WindowContext,
};

use crate::{h_flex, theme::ActiveTheme, v_flex, StyledExt};

pub struct Tooltip {
    title: SharedString,
}

impl Tooltip {
    pub fn new(title: impl Into<SharedString>, cx: &mut WindowContext) -> AnyView {
        cx.new_view(|_cx| Self {
            title: title.into(),
        })
        .into()
    }
}

impl Render for Tooltip {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().child(
            v_flex()
                .m_3()
                .bg(cx.theme().popover)
                .rounded(px(8.))
                .border_1()
                .border_color(cx.theme().border)
                .elevation_2(cx)
                .text_color(cx.theme().popover_foreground)
                .py_1p5()
                .px_2()
                .child(h_flex().gap_4().child(self.title.clone())),
        )
    }
}
