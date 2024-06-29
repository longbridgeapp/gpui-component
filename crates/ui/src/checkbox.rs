use gpui::{
    prelude::FluentBuilder as _, px, svg, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _,
    WindowContext,
};

use crate::{
    disableable::Disableable,
    label::Label,
    selectable::{Selectable, Selection},
    stock::{h_flex, v_flex},
    theme::{ActiveTheme, Colorize as _},
    IconName,
};

type OnClick = Box<dyn Fn(&Selection, &mut WindowContext) + 'static>;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: Selection,
    disabled: bool,
    label: Option<SharedString>,
    on_click: Option<OnClick>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, cx: &mut WindowContext) -> Self {
        Self {
            id: id.into(),
            checked: Selection::Unselected,
            disabled: false,
            label: None,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: Selection) -> Self {
        self.checked = checked;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&Selection, &mut WindowContext) + 'static) -> Self {
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
        self.checked(if selected {
            Selection::Selected
        } else {
            Selection::Unselected
        })
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
            .justify_center()
            .items_center()
            .gap_2()
            .child(
                v_flex()
                    .relative()
                    .border_1()
                    .border_color(color)
                    .rounded_sm()
                    .size_4()
                    .map(|this| match self.checked {
                        Selection::Unselected => this.bg(theme.transparent),
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
                                Selection::Selected => this.path(IconName::Check.path()),
                                Selection::Indeterminate => this.path(IconName::Minus.path()),
                                _ => this,
                            }),
                    ),
            )
            .map(|this| {
                if let Some(label) = self.label {
                    this.child(label).text_color(color)
                } else {
                    this
                }
            })
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_click(move |_, cx| {
                        on_click(&self.checked.inverse(), cx);
                        cx.refresh()
                    })
                },
            )
    }
}
