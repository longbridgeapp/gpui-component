use crate::selectable::Selectable;
use crate::theme::{ActiveTheme, Colorize};
use crate::Icon;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, AnyElement, Div, IntoElement, ParentElement as _, RenderOnce, SharedString, Stateful,
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
        let (text_color, bg_color) = match (self.selected, self.disabled) {
            (true, _) => (cx.theme().foreground, cx.theme().background),
            (false, true) => (cx.theme().foreground.opacity(0.5), cx.theme().muted),
            (false, false) => (cx.theme().muted_foreground, cx.theme().muted),
        };

        self.base
            .flex()
            .items_center()
            .gap_2()
            .h_full()
            .cursor_pointer()
            .py_1()
            .px_3()
            .min_w_16()
            .text_color(text_color)
            .bg(bg_color)
            .when(self.selected, |this| this.rounded(px(6.)))
            .when(self.disabled, |this| this)
            .when_some(self.prefix, |this, prefix| {
                this.child(prefix).text_color(text_color)
            })
            .child(self.label.clone())
            .when_some(self.suffix, |this, suffix| this.child(suffix))
    }
}
