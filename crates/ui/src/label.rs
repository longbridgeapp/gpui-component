use gpui::{
    div, prelude::FluentBuilder as _, px, AbsoluteLength, DefiniteLength, Div, Hsla, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, WindowContext,
};

use crate::theme::ActiveTheme;

#[derive(IntoElement)]
pub struct Label {
    base: Div,
    label: SharedString,
    color: Hsla,
    multiple_lines: bool,
    line_height: Option<DefiniteLength>,
    text_size: Option<AbsoluteLength>,
}

impl Label {
    pub fn new(label: impl Into<SharedString>, cx: &mut WindowContext) -> Self {
        let theme = cx.theme();
        Self {
            base: div().text_color(theme.foreground).text_size(px(14.0)),
            label: label.into(),
            multiple_lines: false,
            color: Hsla::white(),
            line_height: None,
            text_size: None,
        }
    }

    pub fn multiple_lines(mut self) -> Self {
        self.multiple_lines = true;
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
        let label_text = if !self.multiple_lines {
            SharedString::from(self.label.replace('\n', "␤"))
        } else {
            self.label
        };

        self.base
            .child(label_text)
            .map(|this| {
                if let Some(text_size) = self.text_size {
                    this.text_size(text_size)
                } else {
                    this
                }
            })
            .map(|this| match self.line_height {
                Some(line_height) => this.line_height(line_height),
                None => this,
            })
    }
}
