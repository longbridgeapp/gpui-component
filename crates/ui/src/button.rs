use crate::{
    h_flex,
    indicator::Indicator,
    theme::{ActiveTheme, Colorize as _},
    Clickable, Disableable, Icon, Selectable, Size,
};
use gpui::{
    div, prelude::FluentBuilder as _, px, AnyElement, ClickEvent, DefiniteLength, Div, ElementId,
    FocusHandle, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, WindowContext,
};

pub enum ButtonRounded {
    None,
    Small,
    Medium,
    Large,
    Size(Pixels),
}

impl From<Pixels> for ButtonRounded {
    fn from(px: Pixels) -> Self {
        ButtonRounded::Size(px)
    }
}

#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    Outline,
    Ghost,
}

#[derive(IntoElement)]
pub struct Button {
    pub base: Div,
    id: ElementId,
    focus_handle: FocusHandle,
    icon: Option<Icon>,
    label: Option<SharedString>,
    children: Vec<AnyElement>,
    disabled: bool,
    selected: bool,
    width: Option<DefiniteLength>,
    height: Option<DefiniteLength>,
    style: ButtonStyle,
    rounded: ButtonRounded,
    size: Size,
    tooltip: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>>,
    loading: bool,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, cx: &mut WindowContext) -> Self {
        Self {
            base: div(),
            focus_handle: cx.focus_handle(),
            id: id.into(),
            icon: None,
            label: None,
            disabled: false,
            selected: false,
            style: ButtonStyle::Secondary,
            width: None,
            height: None,
            rounded: ButtonRounded::Medium,
            size: Size::Medium,
            tooltip: None,
            on_click: None,
            loading: false,
            children: Vec::new(),
        }
    }

    pub fn primary(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new(id, cx).label(label).style(ButtonStyle::Primary)
    }

    pub fn danger(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new(id, cx).label(label).style(ButtonStyle::Danger)
    }

    pub fn small(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        cx: &mut WindowContext,
    ) -> Self {
        Self::new(id, cx).label(label).size(Size::Small)
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn rounded(mut self, rounded: impl Into<ButtonRounded>) -> Self {
        self.rounded = rounded.into();
        self
    }

    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the icon of the button, if the Button have no label, the button well in Icon Button mode.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
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

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
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

impl Styled for Button {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for Button {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let style: ButtonStyle = self.style;
        let normal_style = style.normal(cx);
        let focused = self.focus_handle.is_focused(cx);
        let icon_size = match self.size {
            Size::Size(v) => Size::Size(v * 0.75),
            _ => self.size,
        };

        self.base
            .id(self.id)
            .track_focus(&self.focus_handle)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .when_some(self.width, |this, width| this.w(width))
            .when_some(self.height, |this, height| this.h(height))
            .map(|this| {
                if self.label.is_none() && self.children.is_empty() {
                    // Icon Button
                    match self.size {
                        Size::Size(px) => this.size(px),
                        Size::XSmall => this.size_5(),
                        Size::Small => this.size_6(),
                        Size::Large | Size::Medium => this.size_8(),
                    }
                } else {
                    // Normal Button
                    match self.size {
                        Size::Size(size) => this.p(size * 0.2),
                        Size::XSmall => this.px_1().py_1().h_5(),
                        Size::Small => this.px_3().py_2().h_6(),
                        _ => this.px_4().py_2().h_8(),
                    }
                }
            })
            .map(|this| match self.rounded {
                ButtonRounded::Small => this.rounded(px(cx.theme().radius * 0.5)),
                ButtonRounded::Medium => this.rounded(px(cx.theme().radius)),
                ButtonRounded::Large => this.rounded(px(cx.theme().radius * 2.0)),
                ButtonRounded::Size(px) => this.rounded(px),
                ButtonRounded::None => this.rounded_none(),
            })
            .when(self.selected, |this| {
                let selected_style = style.selected(cx);
                this.bg(selected_style.bg)
                    .border_color(selected_style.border)
            })
            .when(!self.disabled && !self.selected, |this| {
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
            .when(focused, |this| this.border_color(cx.theme().ring))
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_mouse_down(MouseButton::Left, |_, cx| cx.stop_propagation())
                        .on_click(move |event, cx| {
                            cx.prevent_default();
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
            .child({
                let text_color = if self.disabled {
                    normal_style.fg.opacity(0.6)
                } else {
                    normal_style.fg
                };

                h_flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .text_color(text_color)
                    .when(!self.loading, |this| {
                        this.when_some(self.icon, |this, icon| {
                            this.child(div().text_color(text_color).child(icon.size(icon_size)))
                        })
                    })
                    .when(self.loading, |this| {
                        this.child(Indicator::new().size(self.size).color(text_color))
                    })
                    .when_some(self.label, |this, label| this.child(label))
                    .children(self.children)
                    .map(|this| match self.size {
                        Size::XSmall => this.text_xs(),
                        Size::Small => this.text_sm(),
                        _ => this.text_base(),
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
            ButtonStyle::Outline => cx.theme().transparent,
            ButtonStyle::Ghost => cx.theme().transparent,
        }
    }

    fn text_color(&self, cx: &WindowContext) -> Hsla {
        match self {
            ButtonStyle::Primary => cx.theme().primary_foreground,
            ButtonStyle::Secondary => cx.theme().secondary_foreground,
            ButtonStyle::Danger => cx.theme().destructive_foreground,
            ButtonStyle::Outline => cx.theme().secondary_foreground,
            ButtonStyle::Ghost => cx.theme().secondary_foreground,
        }
    }

    fn border_color(&self, cx: &WindowContext) -> Hsla {
        match self {
            ButtonStyle::Primary => cx.theme().primary,
            ButtonStyle::Secondary => cx.theme().border,
            ButtonStyle::Danger => cx.theme().destructive,
            ButtonStyle::Outline => cx.theme().border,
            ButtonStyle::Ghost => cx.theme().transparent,
        }
    }

    fn normal(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = self.bg_color(cx);
        let border = self.border_color(cx);
        let fg = self.text_color(cx);

        ButtonStyles { bg, border, fg }
    }

    fn hovered(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = match self {
            ButtonStyle::Primary => cx.theme().primary_hover,
            ButtonStyle::Secondary => cx.theme().secondary_hover,
            ButtonStyle::Danger => cx.theme().destructive_hover,
            ButtonStyle::Outline => cx.theme().secondary_hover,
            ButtonStyle::Ghost => cx.theme().secondary,
        };
        let border = self.border_color(cx);
        let fg = self.text_color(cx);

        ButtonStyles { bg, border, fg }
    }

    fn active(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = match self {
            ButtonStyle::Primary => cx.theme().primary_active,
            ButtonStyle::Secondary => cx.theme().secondary_active,
            ButtonStyle::Danger => cx.theme().destructive_active,
            ButtonStyle::Outline => cx.theme().secondary_active,
            ButtonStyle::Ghost => cx.theme().secondary_active,
        };
        let border = self.border_color(cx);
        let fg = self.text_color(cx);

        ButtonStyles { bg, border, fg }
    }

    fn selected(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = match self {
            ButtonStyle::Primary => cx.theme().primary_active,
            ButtonStyle::Secondary => cx.theme().secondary_active,
            ButtonStyle::Danger => cx.theme().destructive_active,
            ButtonStyle::Outline => cx.theme().secondary_active,
            ButtonStyle::Ghost => cx.theme().secondary_active,
        };
        let border = self.border_color(cx);
        let fg = self.text_color(cx);

        ButtonStyles { bg, border, fg }
    }

    fn disabled(&self, cx: &WindowContext) -> ButtonStyles {
        let bg = self.bg_color(cx).grayscale().opacity(0.9);
        let border = self.border_color(cx).grayscale().opacity(0.9);
        let fg = self.text_color(cx).grayscale();

        ButtonStyles { bg, border, fg }
    }
}
