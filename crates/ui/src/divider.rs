use gpui::IntoElement;
use gpui::{div, prelude::FluentBuilder as _, Div, RenderOnce, Styled as _};

use crate::theme::ActiveTheme;
use crate::StyledExt as _;

enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(IntoElement)]
pub struct Divider {
    orientation: Orientation,
}

impl Divider {
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
        }
    }

    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }
}

impl RenderOnce for Divider {
    fn render(self, cx: &mut gpui::WindowContext) -> impl gpui::IntoElement {
        let theme = cx.theme();

        div()
            .map(|this| match self.orientation {
                Orientation::Vertical => this.v_flex().w_0().h_full(),
                Orientation::Horizontal => this.h_flex().h_0().w_full(),
            })
            .border_1()
            .border_color(theme.border)
    }
}
