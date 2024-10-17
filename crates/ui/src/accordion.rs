use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder as _, AnyElement, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    WindowContext,
};

use crate::{h_flex, theme::ActiveTheme as _, v_flex, Icon, IconName, Sizable, Size};

/// An Accordion is a vertically stacked list of items, each of which can be expanded to reveal the content associated with it.
#[derive(IntoElement)]
pub struct Accordion {
    title: AnyElement,
    content: AnyElement,
    expanded: bool,
    size: Size,
    on_toggle_click: Option<Arc<dyn Fn(&bool, &mut WindowContext) + Send + Sync>>,
}

impl Accordion {
    pub fn new() -> Self {
        Self {
            title: SharedString::default().into_any_element(),
            content: SharedString::default().into_any_element(),
            expanded: false,
            on_toggle_click: None,
            size: Size::default(),
        }
    }

    pub fn title(mut self, title: impl IntoElement) -> Self {
        self.title = title.into_any_element();
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = content.into_any_element();
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn on_toggle_click(
        mut self,
        on_toggle_click: impl Fn(&bool, &mut WindowContext) + Send + Sync + 'static,
    ) -> Self {
        self.on_toggle_click = Some(Arc::new(on_toggle_click));
        self
    }
}

impl Sizable for Accordion {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for Accordion {
    fn render(mut self, cx: &mut WindowContext) -> impl IntoElement {
        v_flex()
            .bg(cx.theme().accordion)
            .border_1()
            .rounded_md()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .id("accordion-title")
                    .justify_between()
                    .map(|this| match self.size {
                        Size::Small => this.py_0p5().p_2(),
                        Size::Large => this.py_1p5().p_4(),
                        _ => this.py_1().p_3(),
                    })
                    .cursor_pointer()
                    .when(self.expanded, |this| {
                        this.bg(cx.theme().accordion_active)
                            .text_color(cx.theme().foreground)
                    })
                    .hover(|this| this.bg(cx.theme().accordion_hover))
                    .child(self.title)
                    .child(
                        Icon::new(if self.expanded {
                            IconName::ChevronUp
                        } else {
                            IconName::ChevronDown
                        })
                        .xsmall()
                        .text_color(cx.theme().muted_foreground),
                    )
                    .when_some(self.on_toggle_click.take(), |this, on_toggle_click| {
                        this.on_click({
                            move |_, cx| {
                                on_toggle_click(&!self.expanded, cx);
                            }
                        })
                    }),
            )
            .when(self.expanded, |this| {
                this.child(
                    div()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .map(|this| match self.size {
                            Size::Small => this.p_2(),
                            Size::Large => this.p_4(),
                            _ => this.p_3(),
                        })
                        .child(self.content),
                )
            })
    }
}
