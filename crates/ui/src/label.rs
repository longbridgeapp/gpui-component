use gpui::{
    div, prelude::FluentBuilder, Div, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    WindowContext,
};

use crate::{h_flex, theme::ActiveTheme};

#[derive(Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(IntoElement)]
pub struct Label {
    base: Div,
    label: SharedString,
    multiple_lines: bool,
    align: TextAlign,
}

impl Label {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            label: label.into(),
            multiple_lines: true,
            align: TextAlign::default(),
        }
    }

    pub fn multiple_lines(mut self) -> Self {
        self.multiple_lines = true;
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl Styled for Label {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Label {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let text = if !self.multiple_lines {
            SharedString::from(self.label.replace('\n', "␤"))
        } else {
            self.label
        };

        h_flex()
            .map(|this| match self.align {
                TextAlign::Left => this.justify_start(),
                TextAlign::Center => this.justify_center(),
                TextAlign::Right => this.justify_end(),
            })
            .text_color(cx.theme().foreground)
            .child(self.base.child(text))
    }
}
