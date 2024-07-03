use gpui::{
    div, px, AnchorCorner, AppContext, DismissEvent, Element, EventEmitter, FocusHandle,
    FocusableView, IntoElement, MouseButton, ParentElement as _, Render, Styled as _, View,
    ViewContext, VisualContext, WindowContext,
};
use ui::{
    button::{Button, ButtonSize},
    divider::Divider,
    h_flex,
    input::TextInput,
    popover::{Popover, PopoverContent},
    v_flex, Clickable,
};

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
        v_flex()
            .gap_4()
            .size_full()
            .child("This is a form container.")
            .child(self.input1.clone())
            .child(
                Button::primary("submit", "Submit")
                    .on_click(cx.listener(|_, _, cx| cx.emit(DismissEvent))),
            )
    }
}

pub struct PopoverStory {
    form: View<Form>,
}

impl PopoverStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let form = Form::new(cx);
        Self { form }
    }
}

impl Render for PopoverStory {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let form = self.form.clone();

        v_flex()
            .p_4()
            .size_full()
            .gap_6()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_4().child(
                            Popover::new("info-top-left")
                                .trigger(Button::new("info-top-left").label("Top Left"))
                                .content(|cx| {
                                    PopoverContent::new(cx, |_| {
                                        v_flex()
                                            .gap_4()
                                            .child("Hello, this is a Popover.")
                                            .child(Divider::horizontal())
                                            .child(
                                                Button::new("info1")
                                                    .label("Yes")
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
                            .anchor(AnchorCorner::TopRight)
                            .trigger(Button::new("info-top-right").label("Top Right"))
                            .content(|cx| {
                                PopoverContent::new(cx, |_| {
                                    v_flex()
                                        .gap_4()
                                        .child("Hello, this is a Popover on the Top Right.")
                                        .child(Divider::horizontal())
                                        .child(
                                            Button::new("info1")
                                                .label("Yes")
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
                                .anchor(AnchorCorner::BottomLeft)
                                .trigger(
                                    Button::new("pop").label("Popup with Form").width(px(300.)),
                                )
                                .content(move |_| form.clone()),
                        )
                        .child(
                            Popover::new("info-bottom-right")
                                .anchor(AnchorCorner::BottomRight)
                                .mouse_button(MouseButton::Right)
                                .trigger(
                                    Button::new("pop")
                                        .label("Mouse Right Click")
                                        .width(px(300.)),
                                )
                                .content(|cx| {
                                    PopoverContent::new(cx, |_| {
                                        v_flex()
                                            .gap_4()
                                            .child("Hello, this is a Popover on the Bottom Right.")
                                            .child(Divider::horizontal())
                                            .child(
                                                Button::new("info1")
                                                    .label("Yes")
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
