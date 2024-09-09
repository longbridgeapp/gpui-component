use gpui::{
    div, prelude::FluentBuilder as _, px, Axis, Div, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled,
};

use crate::theme::ActiveTheme;

#[derive(IntoElement)]
pub struct Divider {
    base: Div,
    label: Option<SharedString>,
    axis: Axis,
}

impl Divider {
    pub fn vertical() -> Self {
        Self {
            base: div(),
            axis: Axis::Vertical,
            label: None,
        }
    }

    pub fn horizontal() -> Self {
        Self {
            base: div(),
            axis: Axis::Horizontal,
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Styled for Divider {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Divider {
    fn render(self, cx: &mut gpui::WindowContext) -> impl gpui::IntoElement {
        let theme = cx.theme();

        self.base
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .map(|this| match self.axis {
                Axis::Vertical => this.h_full(),
                Axis::Horizontal => this.w_full(),
            })
            .child(
                div()
                    .absolute()
                    .map(|this| match self.axis {
                        Axis::Vertical => this.w(px(1.)).h_full(),
                        Axis::Horizontal => this.h(px(1.)).w_full(),
                    })
                    .bg(cx.theme().border),
            )
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .px_2()
                        .py_1()
                        .mx_auto()
                        .text_xs()
                        .bg(cx.theme().background)
                        .text_color(theme.muted_foreground)
                        .child(label),
                )
            })
    }
}
