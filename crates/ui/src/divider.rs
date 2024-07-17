use gpui::{div, prelude::FluentBuilder as _, RenderOnce};
use gpui::{Div, IntoElement, ParentElement, Styled};

use crate::theme::ActiveTheme;
use crate::StyledExt as _;

enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(IntoElement)]
pub struct Divider {
    base: Div,
    orientation: Orientation,
}

impl Divider {
    pub fn vertical() -> Self {
        Self {
            base: div(),
            orientation: Orientation::Vertical,
        }
    }

    pub fn horizontal() -> Self {
        Self {
            base: div(),
            orientation: Orientation::Horizontal,
        }
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
            .map(|this| match self.orientation {
                Orientation::Vertical => this.v_flex().h_full(),
                Orientation::Horizontal => this.h_flex().w_full(),
            })
            .child(
                div()
                    .map(|this| match self.orientation {
                        Orientation::Vertical => this.v_flex().w_0().h_full().border_l_1(),
                        Orientation::Horizontal => this.h_flex().h_0().w_full().border_b_1(),
                    })
                    .border_color(theme.border),
            )
    }
}
