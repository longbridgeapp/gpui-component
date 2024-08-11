use gpui::{div, prelude::FluentBuilder as _, RenderOnce};
use gpui::{Axis, Div, IntoElement, ParentElement, SharedString, Styled};

use crate::theme::ActiveTheme;
use crate::StyledExt as _;

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
            .map(|this| match self.axis {
                Axis::Vertical => this.flex().flex_row().items_center().h_full(),
                Axis::Horizontal => this.h_flex().w_full(),
            })
            .child(
                div()
                    .absolute()
                    .map(|this| match self.axis {
                        Axis::Vertical => this.v_flex().w_0().h_full().border_l_1(),
                        Axis::Horizontal => this.h_flex().h_0().w_full().border_b_1(),
                    })
                    .border_color(cx.theme().border),
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
