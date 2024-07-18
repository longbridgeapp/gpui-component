use gpui::{
    div, prelude::FluentBuilder, svg, CursorStyle, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, WindowContext,
};

use crate::{
    h_flex,
    theme::{ActiveTheme, Colorize},
    IconName,
};

#[derive(IntoElement)]
pub struct Radio {
    id: ElementId,
    label: Option<SharedString>,
    selected: bool,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&bool, &mut WindowContext) + 'static>>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            selected: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&bool, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Radio {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let color = if self.disabled {
            cx.theme().primary.opacity(0.5)
        } else {
            cx.theme().primary
        };

        h_flex()
            .id(self.id)
            .gap_x_2()
            .cursor(CursorStyle::PointingHand)
            .text_color(color)
            .child(
                div()
                    .relative()
                    .w_3p5()
                    .h_3p5()
                    .rounded_full()
                    .border_1()
                    .border_color(color)
                    .mt_neg_0p5()
                    .child(
                        svg()
                            .absolute()
                            .top_px()
                            .left_px()
                            .size_2p5()
                            .text_color(color)
                            .map(|this| match self.selected {
                                true => this.path(IconName::Check.path()),
                                false => this,
                            }),
                    ),
            )
            .when_some(self.label, |this, label| this.child(label))
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    this.on_click(move |_event, cx| {
                        on_click(&!self.selected, cx);
                    })
                },
            )
    }
}
