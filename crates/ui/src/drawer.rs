use std::{cell::RefCell, ops::Deref, rc::Rc, time::Duration};

use gpui::{
    canvas, div, hsla, prelude::FluentBuilder as _, px, Animation, AnimationExt as _, AnyElement,
    ClickEvent, DefiniteLength, DismissEvent, Div, EventEmitter, InteractiveElement as _,
    IntoElement, ManagedView, MouseButton, ParentElement, Render, RenderOnce, Styled, View,
    ViewContext, WindowContext,
};

use crate::{
    button::Button, theme::ActiveTheme, v_flex, IconName, Placement, Selectable, StyledExt as _,
};

pub fn drawer() -> Drawer {
    Drawer::new()
}

#[derive(IntoElement)]
pub struct Drawer {
    base: Div,
    placement: Placement,
    size: DefiniteLength,
    resizable: bool,
    open: bool,
    on_close: Rc<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>,
}

impl Drawer {
    pub fn new() -> Self {
        Self {
            base: div(),
            placement: Placement::Right,
            size: DefiniteLength::Absolute(px(350.).into()),
            open: false,
            resizable: true,
            on_close: Rc::new(|_, _| {}),
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Sets the size of the drawer, default is 350px.
    pub fn size(mut self, size: impl Into<DefiniteLength>) -> Self {
        self.size = size.into();
        self
    }

    /// Sets the placement of the drawer, default is `Placement::Right`.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Sets whether the drawer is resizable, default is `true`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn on_close(
        mut self,
        on_close: impl Fn(&ClickEvent, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_close = Rc::new(on_close);
        self
    }
}

impl EventEmitter<DismissEvent> for Drawer {}
impl ParentElement for Drawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl RenderOnce for Drawer {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let bounds = cx.content_mask().bounds;

        div().when(self.open, |this| {
            this.occlude()
                .absolute()
                .flex()
                .size_full()
                .h(bounds.size.height)
                .w(bounds.size.width)
                .top_0()
                .bg(hsla(0., 0., 0., 0.25))
                .debug_red()
                .on_mouse_down(MouseButton::Left, {
                    let on_close = self.on_close.clone();
                    move |_, cx| {
                        on_close(&ClickEvent::default(), cx);
                    }
                })
                .child(
                    v_flex()
                        .id("")
                        .absolute()
                        .occlude()
                        .bg(cx.theme().background)
                        .border_1()
                        .border_color(cx.theme().border)
                        .shadow_xl()
                        .debug_yellow()
                        .map(|this| {
                            // Set the size of the drawer.
                            if self.placement.is_horizontal() {
                                this.w_full().h(self.size)
                            } else {
                                this.h_full().w(self.size)
                            }
                        })
                        .map(|this| match self.placement {
                            Placement::Top => this.top_0().left_0().right_0(),
                            Placement::Right => this.top_0().right_0().bottom_0(),
                            Placement::Bottom => this.bottom_0().left_0().right_0(),
                            Placement::Left => this.top_0().left_0().bottom_0(),
                        })
                        .child(div().absolute().top_4().right_4().child(
                            Button::new("close", cx).icon(IconName::Close).on_click({
                                let on_close = self.on_close.clone();
                                move |event, cx| {
                                    on_close(event, cx);
                                }
                            }),
                        ))
                        .child(self.base)
                        .with_animation(
                            "slide",
                            Animation::new(Duration::from_secs_f64(0.15)),
                            move |this, delta| {
                                let y = px(-100.) + delta * px(100.);
                                this.map(|this| match self.placement {
                                    Placement::Top => this.top(y),
                                    Placement::Right => this.right(y),
                                    Placement::Bottom => this.bottom(y),
                                    Placement::Left => this.left(y),
                                })
                                .opacity((1.0 * delta + 0.3).min(1.0))
                            },
                        ),
                )
        })
    }
}
