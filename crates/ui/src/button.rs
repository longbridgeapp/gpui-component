use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, DefiniteLength, Div, ElementId, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, WindowContext,
};

use crate::{
    disableable::{Clickable, Disableable, Selectable},
    label::Label,
    theme::{ActiveTheme, Colorize as _, Theme, ThemeMode},
};

pub enum ButtonRounded {
    None,
    Small,
    Medium,
    Large,
}

pub enum ButtonSize {
    Small,
    Medium,
}

pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
}

#[derive(IntoElement)]
pub struct Button {
    pub base: Div,
    id: ElementId,
    label: SharedString,
    disabled: bool,
    selected: bool,
    width: Option<DefiniteLength>,
    height: Option<DefiniteLength>,
    style: ButtonStyle,
    rounded: ButtonRounded,
    size: ButtonSize,
    tooltip: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            id: id.into(),
            label: label.into(),
            disabled: false,
            selected: false,
            style: ButtonStyle::Secondary,
            width: None,
            height: None,
            rounded: ButtonRounded::Medium,
            size: ButtonSize::Medium,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn rounded(mut self, rounded: ButtonRounded) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
}

impl Disableable for Button {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Button {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Clickable for Button {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();
        let style: ButtonStyle = self.style;
        let normal_style = style.normal(cx);

        self.base
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .map(|this| match self.size {
                ButtonSize::Small => this.px_3().py_2().h_6(),
                ButtonSize::Medium => this.px_4().py_2().h_8(),
            })
            .map(|this| match self.rounded {
                ButtonRounded::Small => this.rounded_sm(),
                ButtonRounded::Medium => this.rounded_md(),
                ButtonRounded::Large => this.rounded_lg(),
                ButtonRounded::None => this.rounded_none(),
            })
            .when(!self.disabled, |this| {
                this.hover(|this| {
                    let hover_style = style.hovered(cx);
                    this.bg(hover_style.bg).border_color(hover_style.border)
                })
                .active(|this| {
                    let active_style = style.active(cx);
                    this.bg(active_style.bg).border_color(active_style.border)
                })
                .border_color(normal_style.border)
                .bg(normal_style.bg)
            })
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_mouse_down(MouseButton::Left, |_, cx| cx.prevent_default())
                        .on_click(move |event, cx| {
                            cx.stop_propagation();
                            (on_click)(event, cx)
                        })
                },
            )
            .when(self.disabled, |this| {
                let disabled_style = style.disabled(cx);
                this.cursor_not_allowed()
                    .bg(disabled_style.bg)
                    .border_color(disabled_style.border)
            })
            .border_1()
            .map(|this| match theme.mode {
                ThemeMode::Light => this.shadow_sm(),
                ThemeMode::Dark => this,
            })
            .child({
                let text_color = if self.disabled {
                    normal_style.fg.opacity(0.6)
                } else {
                    normal_style.fg
                };

                Label::new(self.label, cx)
                    .text_color(text_color)
                    .map(|this| match self.size {
                        ButtonSize::Small => this.text_sm(),
                        ButtonSize::Medium => this.text_base(),
                    })
            })
    }
}

struct ButtonStyles {
    bg: Hsla,
    border: Hsla,
    fg: Hsla,
}

impl ButtonStyle {
    fn bg_color(&self, cx: &WindowContext) -> Hsla {
        match self {
            ButtonStyle::Primary => cx.theme().primary,
            ButtonStyle::Secondary => cx.theme().secondary,
            ButtonStyle::Danger => cx.theme().destructive,
        }
    }

    fn text_color(&self, cx: &WindowContext) -> Hsla {
        match self {
            ButtonStyle::Primary => cx.theme().primary_foreground,
            ButtonStyle::Secondary => cx.theme().secondary_foreground,
            ButtonStyle::Danger => cx.theme().destructive_foreground,
        }
    }

    fn border_color(&self, cx: &WindowContext) -> Hsla {
        match self {
            ButtonStyle::Primary => cx.theme().primary.darken(0.05),
            ButtonStyle::Secondary => cx.theme().border,
            ButtonStyle::Danger => cx.theme().destructive.darken(0.05),
        }
    }

    fn normal(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = self.bg_color(cx);
        let border = self.border_color(cx);
        let fg = self.text_color(cx);

        ButtonStyles { bg, border, fg }
    }

    fn hovered(&self, cx: &WindowContext) -> ButtonStyles {
        // Hover color = color/90
        let bg = self.bg_color(cx).divide(0.9);
        let border = self.border_color(cx).divide(0.9);
        let fg = self.text_color(cx).divide(0.9);

        ButtonStyles { bg, border, fg }
    }

    fn active(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = self.bg_color(cx).darken(0.05);
        let border = self.border_color(cx).darken(0.05);
        let fg = self.text_color(cx).darken(0.05);

        ButtonStyles { bg, border, fg }
    }

    fn disabled(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = self.bg_color(cx).grayscale();
        let border = self.border_color(cx).grayscale();
        let fg = self.text_color(cx).grayscale();

        ButtonStyles { bg, border, fg }
    }
}
