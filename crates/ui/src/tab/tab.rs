use crate::selectable::Selectable;
use crate::theme::{ActiveTheme, Colorize};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, AnyElement, Div, IntoElement, ParentElement as _, RenderOnce, SharedString, Stateful,
    StatefulInteractiveElement, WindowContext,
};
use gpui::{InteractiveElement, Styled as _};

#[derive(IntoElement)]
pub struct Tab {
    base: Stateful<Div>,
    label: SharedString,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    disabled: bool,
    selected: bool,
}

impl Tab {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            base: div().id(id.into()),
            label: label.into(),
            disabled: false,
            selected: false,
            prefix: None,
            suffix: None,
        }
    }

    /// Set the left side of the tab
    pub fn prefix(mut self, prefix: impl Into<AnyElement>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the right side of the tab
    pub fn suffix(mut self, suffix: impl Into<AnyElement>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
}

impl Selectable for Tab {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl InteractiveElement for Tab {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Tab {}

impl RenderOnce for Tab {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();

        self.base
            .flex()
            .items_center()
            .gap_2()
            .h_full()
            .cursor_pointer()
            .py_1()
            .px_3()
            .min_w_16()
            .text_color(theme.muted_foreground)
            .bg(theme.muted)
            .when(self.selected, |this| {
                this.text_color(theme.foreground)
                    .bg(theme.background)
                    .rounded_sm()
            })
            .when(self.disabled, |this| {
                this.text_color(theme.foreground.opacity(0.5))
            })
            .when_some(self.prefix, |this, prefix| this.child(prefix))
            .child(self.label.clone())
            .when_some(self.suffix, |this, suffix| this.child(suffix))
    }
}
