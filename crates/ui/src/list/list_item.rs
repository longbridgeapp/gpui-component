use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, Div, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, RenderOnce, SharedString, Stateful,
    StatefulInteractiveElement as _, Style, Styled, WindowContext,
};

use crate::{h_flex, theme::ActiveTheme, Disableable, IconName, Selectable};

#[derive(IntoElement)]
pub struct ListItem {
    base: Stateful<Div>,
    disabled: bool,
    selected: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>>,
    on_secondary_mouse_down: Option<Box<dyn Fn(&MouseDownEvent, &mut WindowContext) + 'static>>,
}

impl ListItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            base: h_flex().id(id.into()),
            disabled: false,
            selected: false,
            on_click: None,
            on_secondary_mouse_down: None,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_secondary_mouse_down(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_secondary_mouse_down = Some(Box::new(handler));
        self
    }
}

impl Disableable for ListItem {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for ListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Styled for ListItem {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for ListItem {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        self.base
            .w_full()
            .relative()
            .gap_x_2()
            .text_base()
            .text_color(cx.theme().foreground)
            .when_some(self.on_click, |this, on_click| {
                this.cursor_pointer().on_click(on_click)
            })
            // Right click
            .when_some(self.on_secondary_mouse_down, |this, on_mouse_down| {
                this.on_mouse_down(MouseButton::Right, move |ev, cx| (on_mouse_down)(ev, cx))
            })
            .when(!self.selected, |this| {
                this.hover(|this| this.bg(cx.theme().accent))
            })
            .when(self.selected, |this| this.bg(cx.theme().accent))
            .map(|this| {
                if self.selected {
                    this.child(IconName::Check)
                } else {
                    this.child(div())
                }
            })
    }
}
