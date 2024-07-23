use gpui::{
    div, prelude::FluentBuilder as _, AnyElement, ClickEvent, Div, ElementId, InteractiveElement,
    IntoElement, MouseButton, MouseDownEvent, ParentElement, RenderOnce, Stateful,
    StatefulInteractiveElement as _, Styled, WindowContext,
};
use smallvec::SmallVec;

use crate::{h_flex, theme::ActiveTheme, Disableable, Icon, IconName, Selectable};

#[derive(IntoElement)]
pub struct ListItem {
    base: Stateful<Div>,
    disabled: bool,
    selected: bool,
    check_icon: Option<Icon>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>>,
    on_secondary_mouse_down: Option<Box<dyn Fn(&MouseDownEvent, &mut WindowContext) + 'static>>,
    suffix: Option<Box<dyn Fn(&mut WindowContext) -> AnyElement + 'static>>,
    children: SmallVec<[AnyElement; 2]>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: h_flex().id(id.into()).gap_x_1().py_1().px_2().text_base(),
            disabled: false,
            selected: false,
            on_click: None,
            on_secondary_mouse_down: None,
            check_icon: None,
            suffix: None,
            children: SmallVec::new(),
        }
    }

    pub fn check_icon(mut self, icon: IconName) -> Self {
        self.check_icon = Some(Icon::new(icon));
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

    /// Set the suffix element of the input field, for example a clear button.
    pub fn suffix<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut WindowContext) -> E + 'static,
        E: IntoElement,
    {
        self.suffix = Some(Box::new(move |cx| builder(cx).into_any_element()));
        self
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
        self.children.extend(elements);
    }
}

impl RenderOnce for ListItem {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        self.base
            .text_color(cx.theme().foreground)
            .relative()
            .items_center()
            .justify_between()
            .when_some(self.on_click, |this, on_click| {
                if !self.disabled {
                    this.cursor_pointer().on_click(on_click)
                } else {
                    this
                }
            })
            .when(self.selected, |this| this.bg(cx.theme().list_item_active))
            .when(!self.selected && !self.disabled, |this| {
                this.hover(|this| this.bg(cx.theme().list_item_hover))
            })
            // Right click
            .when_some(self.on_secondary_mouse_down, |this, on_mouse_down| {
                if !self.disabled {
                    this.on_mouse_down(MouseButton::Right, move |ev, cx| (on_mouse_down)(ev, cx))
                } else {
                    this
                }
            })
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_1()
                    .child(div().w_full().children(self.children))
                    .when_some(self.check_icon, |this, icon| {
                        this.child(
                            div()
                                .w_5()
                                .items_center()
                                .justify_center()
                                .when(self.selected, |this| {
                                    this.child(icon.text_color(cx.theme().muted_foreground))
                                }),
                        )
                    }),
            )
            .when_some(self.suffix, |this, suffix| this.child(suffix(cx)))
    }
}
