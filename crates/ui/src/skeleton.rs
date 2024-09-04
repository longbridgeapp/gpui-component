use std::time::Duration;

use gpui::{
    bounce, div, ease_in_out, Animation, AnimationExt, Div, IntoElement, ParentElement as _,
    RenderOnce, Styled,
};

use crate::theme::ActiveTheme;

#[derive(IntoElement)]
pub struct Skeleton {
    base: Div,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            base: div().w_full().h_4().rounded_md(),
        }
    }
}

impl Styled for Skeleton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Skeleton {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        div().child(
            self.base.bg(cx.theme().skeleton).with_animation(
                "skeleton",
                Animation::new(Duration::from_secs(2))
                    .repeat()
                    .with_easing(bounce(ease_in_out)),
                move |this, delta| {
                    let v = 1.0 - delta * 0.5;
                    this.opacity(v)
                },
            ),
        )
    }
}
