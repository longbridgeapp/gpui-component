use gpui::{
    div, prelude::FluentBuilder, px, relative, IntoElement, ParentElement, RenderOnce, Styled,
    WindowContext,
};

use crate::{
    h_flex,
    theme::{ActiveTheme, Colorize},
};

#[derive(IntoElement)]
pub struct Progress {
    value: f32,
    height: f32,
}

impl Progress {
    pub fn new() -> Self {
        Progress {
            value: Default::default(),
            height: 8.,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let rounded = px(self.height / 2.);
        let relative_w = relative(match self.value {
            v if v < 0. => 0.,
            v if v > 100. => 1.,
            v => v / 100.,
        });

        h_flex()
            .h(px(self.height))
            .child(
                div()
                    .map(|this| match self.value {
                        v if v >= 100. => this.rounded(rounded),
                        _ => this.rounded_l(rounded),
                    })
                    .h_full()
                    .w(relative_w)
                    .bg(cx.theme().primary),
            )
            .child(
                div()
                    .map(|this| match self.value {
                        v if v <= 0. => this.rounded(rounded),
                        _ => this.rounded_r(rounded),
                    })
                    .h_full()
                    .flex_1()
                    .bg(cx.theme().primary.opacity(0.2)),
            )
    }
}
