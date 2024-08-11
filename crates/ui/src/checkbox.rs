use gpui::{
    div, prelude::FluentBuilder as _, relative, svg, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _,
    WindowContext,
};

use crate::{
    h_flex,
    theme::{ActiveTheme, Colorize as _},
    v_flex, Disableable, IconName, Selectable,
};

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&bool, &mut WindowContext) + 'static>>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            label: None,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&bool, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Disableable for Checkbox {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Checkbox {
    fn selected(self, selected: bool) -> Self {
        self.checked(selected)
    }
}

impl RenderOnce for Checkbox {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();

        let group_id = format!("checkbox_group_{:?}", self.id);

        let (color, icon_color) = if self.disabled {
            (
                theme.primary.opacity(0.5),
                theme.primary_foreground.opacity(0.5),
            )
        } else {
            (theme.primary, theme.primary_foreground)
        };

        h_flex()
            .id(self.id)
            .group(group_id.clone())
            .gap_2()
            .items_start()
            .child(
                v_flex()
                    .relative()
                    .border_1()
                    .border_color(color)
                    .rounded_sm()
                    .size_4()
                    .flex_shrink_0()
                    .map(|this| match self.checked {
                        false => this.bg(theme.transparent),
                        _ => this.bg(color),
                    })
                    .group_hover(group_id, |this| {
                        if self.disabled {
                            return this;
                        }

                        this.border_color(theme.primary.divide(0.9))
                    })
                    .child(
                        svg()
                            .absolute()
                            .top_px()
                            .left_px()
                            .size_3()
                            .text_color(icon_color)
                            .map(|this| match self.checked {
                                true => this.path(IconName::Check.path()),
                                _ => this,
                            }),
                    ),
            )
            .map(|this| {
                if let Some(label) = self.label {
                    this.child(
                        div()
                            .w_full()
                            .overflow_hidden()
                            .line_height(relative(1.))
                            .child(label),
                    )
                    .text_color(color)
                } else {
                    this
                }
            })
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_click(move |_, cx| {
                        let checked = !self.checked;
                        on_click(&checked, cx);
                        cx.refresh()
                    })
                },
            )
    }
}
