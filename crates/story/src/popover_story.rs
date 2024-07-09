use gpui::{
    actions, div, px, AnchorCorner, AppContext, DismissEvent, Element, EventEmitter, FocusHandle,
    FocusableView, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement as _, Render, Styled as _, View, ViewContext, VisualContext, WindowContext,
};
use ui::{
    button::{Button, ButtonSize},
    divider::Divider,
    h_flex,
    input::TextInput,
    popover::{Popover, PopoverContent},
    popup_menu::PopupMenu,
    v_flex, Clickable, IconName,
};

actions!(popover_story, [Copy, Paste, Cut, SearchAll]);

struct Form {
    input1: View<TextInput>,
}

impl Form {
    fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self {
            input1: cx.new_view(TextInput::new),
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
            .p_4()
            .size_full()
            .child("This is a form container.")
            .child(self.input1.clone())
            .child(
                Button::primary("submit", "Submit", cx)
                    .on_click(cx.listener(|_, _, cx| cx.emit(DismissEvent))),
            )
    }
}

pub struct PopoverStory {
    focus_handle: FocusHandle,
    form: View<Form>,
    message: String,
}

impl PopoverStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let form = Form::new(cx);
        Self {
            form,
            focus_handle: cx.focus_handle(),
            message: "".to_string(),
        }
    }

    fn on_copy(&mut self, _: &Copy, cx: &mut ViewContext<Self>) {
        self.message = "You have clicked copy".to_string();
    }
    fn on_cut(&mut self, _: &Cut, _: &mut ViewContext<Self>) {
        self.message = "You have clicked cut".to_string();
    }
    fn on_paste(&mut self, _: &Paste, _: &mut ViewContext<Self>) {
        self.message = "You have clicked paste".to_string();
    }
    fn on_search_all(&mut self, _: &SearchAll, _: &mut ViewContext<Self>) {
        self.message = "You have clicked search all".to_string();
    }
}

impl FocusableView for PopoverStory {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopoverStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let form = self.form.clone();
        let _focused = self.focus_handle.is_focused(cx);
        let focus_handle = self.focus_handle.clone();

        v_flex()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_copy))
            .on_action(cx.listener(Self::on_cut))
            .on_action(cx.listener(Self::on_paste))
            .on_action(cx.listener(Self::on_search_all))
            .p_4()
            .mb_5()
            .size_full()
            .min_h(px(400.))
            .on_any_mouse_down(cx.listener(|this, _: &MouseDownEvent, cx| {
                cx.focus(&this.focus_handle);
            }))
            .gap_6()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_4().child(
                            Popover::new("info-top-left")
                                .trigger(Button::new("info-top-left", cx).label("Top Left"))
                                .content(|cx| {
                                    PopoverContent::new(cx, |cx| {
                                        v_flex()
                                            .gap_4()
                                            .child("Hello, this is a Popover.")
                                            .child(Divider::horizontal())
                                            .child(
                                                Button::new("info1", cx)
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
                            .trigger(Button::new("info-top-right", cx).label("Top Right"))
                            .content(|cx| {
                                PopoverContent::new(cx, |cx| {
                                    v_flex()
                                        .gap_4()
                                        .child("Hello, this is a Popover on the Top Right.")
                                        .child(Divider::horizontal())
                                        .child(
                                            Button::new("info1", cx)
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
                h_flex()
                    .child(
                        Popover::new("popup-menu")
                            .trigger(Button::new("popup-menu-1", cx).icon(IconName::Info))
                            .content(move |cx| {
                                let focus_handle = focus_handle.clone();
                                PopupMenu::build(cx, |mut this, _| {
                                    this.content(focus_handle)
                                        .menu("Copy", Box::new(Copy))
                                        .menu("Cut", Box::new(Cut))
                                        .menu("Paste", Box::new(Paste))
                                        .separator()
                                        .menu_with_icon(
                                            IconName::Search,
                                            "Search",
                                            Box::new(SearchAll),
                                        );

                                    this
                                })
                            }),
                    )
                    .child(self.message.clone()),
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
                                    Button::new("pop", cx)
                                        .label("Popup with Form")
                                        .width(px(300.)),
                                )
                                .content(move |_| form.clone()),
                        )
                        .child(
                            Popover::new("info-bottom-right")
                                .anchor(AnchorCorner::BottomRight)
                                .mouse_button(MouseButton::Right)
                                .trigger(
                                    Button::new("pop", cx)
                                        .label("Mouse Right Click")
                                        .width(px(300.)),
                                )
                                .content(|cx| {
                                    PopoverContent::new(cx, |cx| {
                                        v_flex()
                                            .gap_4()
                                            .child("Hello, this is a Popover on the Bottom Right.")
                                            .child(Divider::horizontal())
                                            .child(
                                                Button::new("info1", cx)
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
