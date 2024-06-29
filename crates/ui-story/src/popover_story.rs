use gpui::{
    div, px, AppContext, DismissEvent, Div, Element, EventEmitter, FocusHandle, FocusableView,
    IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext, VisualContext,
    WindowContext,
};
use ui::{
    button::{Button, ButtonSize},
    divider::Divider,
    h_flex,
    input::TextInput,
    popover::{Popover, PopoverContent},
    v_flex,
};

use crate::story_case;

struct Form {
    input1: View<TextInput>,
}

impl Form {
    fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self {
            input1: cx.new_view(|cx| TextInput::new(cx)),
        })
    }
}

impl FocusableView for Form {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.input1.focus_handle(cx)
    }
}

impl EventEmitter<DismissEvent> for Form {}

impl Render for Form {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p_5()
            .child("This is a form container.")
            .child(self.input1.clone())
    }
}

pub struct PopoverStory {
    form: View<Form>,
}

impl PopoverStory {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let form = Form::new(cx);
        Self { form }
    }
}

impl Render for PopoverStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let form = self.form.clone();

        story_case(
            "Popover",
            "Displays rich content in a portal, triggered by a button.",
        )
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    v_flex().gap_4().child(
                        Popover::new("info-top-left")
                            .trigger(Button::new("info-top-left", "Top Left"))
                            .content(|cx| {
                                PopoverContent::new(cx, |_| {
                                    v_flex()
                                        .gap_4()
                                        .child("Hello, this is a Popover.")
                                        .child(Divider::horizontal())
                                        .child(
                                            Button::new("info1", "Yes")
                                                .width(px(80.))
                                                .size(ButtonSize::Small),
                                        )
                                        .into_any()
                                })
                            }),
                    ),
                )
                .child(
                    Popover::new("info-top-right")
                        .trigger(Button::new("info-top-right", "Top Right"))
                        .content(|cx| {
                            PopoverContent::new(cx, |_| {
                                v_flex()
                                    .gap_4()
                                    .child("Hello, this is a Popover.")
                                    .child(Divider::horizontal())
                                    .child(
                                        Button::new("info1", "Yes")
                                            .width(px(80.))
                                            .size(ButtonSize::Small),
                                    )
                                    .into_any()
                            })
                        }),
                ),
        )
        .child(
            div().absolute().bottom_4().left_0().w_full().h_10().child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Popover::new("info-bottom-left")
                            .trigger(Button::new("pop", "Popup with Form").width(px(300.)))
                            .content(move |_| form.clone()),
                    )
                    .child(
                        Popover::new("info-bottom-right")
                            .trigger(Button::new("pop", "Bottom Right").width(px(300.)))
                            .content(|cx| {
                                PopoverContent::new(cx, |_| {
                                    v_flex()
                                        .gap_4()
                                        .child("Hello, this is a Popover.")
                                        .child(Divider::horizontal())
                                        .child(
                                            Button::new("info1", "Yes")
                                                .width(px(80.))
                                                .size(ButtonSize::Small),
                                        )
                                        .into_any()
                                })
                            }),
                    ),
            ),
        )
    }
}
