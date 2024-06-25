use crate::{
    theme::{ActiveTheme, Colorize},
    Disableable,
};
use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, Div, InteractiveElement, IntoElement,
    ParentElement as _, RenderOnce, SharedString, Stateful, StatefulInteractiveElement,
    Styled as _, WindowContext,
};

type OnClick = Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>;

#[derive(IntoElement)]
pub struct Switch {
    base: Stateful<Div>,
    checked: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_click: Option<OnClick>,
}

impl Switch {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            base: div().id(id.into()),
            checked: false,
            disabled: false,
            label: None,
            on_click: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Disableable for Switch {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        let theme = cx.theme();

        let (bg, toggle_bg) = match self.checked {
            true => (theme.primary, theme.background),
            false => (theme.input, theme.background),
        };

        let (bg, toggle_bg) = match self.disabled {
            true => (bg.opacity(0.3), toggle_bg.opacity(0.8)),
            false => (bg, toggle_bg),
        };

        self.base
            .w_11()
            .h_6()
            .flex()
            .items_center()
            .rounded_xl()
            .border_2()
            .border_color(theme.transparent)
            .bg(bg)
            .when(!self.disabled, |this| this.cursor_pointer())
            .map(|this| match self.checked {
                true => this.flex_row_reverse(),
                false => this,
            })
            .child(div().h_4().w_4().rounded_full().bg(toggle_bg))
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| this.on_click(move |ev, cx| on_click(ev, cx)),
            )
    }
}
