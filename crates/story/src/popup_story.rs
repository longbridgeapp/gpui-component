use gpui::{
    actions, div, px, AnchorCorner, AppContext, DismissEvent, Element, EventEmitter, FocusHandle,
    FocusableView, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement as _, Render, Styled as _, View, ViewContext, VisualContext, WindowContext,
};
use ui::{
    button::Button,
    context_menu::ContextMenuExt,
    divider::Divider,
    h_flex,
    input::TextInput,
    popover::{Popover, PopoverContent},
    popup_menu::PopupMenu,
    prelude::FluentBuilder,
    switch::Switch,
    v_flex, Clickable as _, IconName, Sizable,
};

actions!(
    popover_story,
    [Copy, Paste, Cut, SearchAll, ToggleWindowMode]
);

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
                Button::new("submit", cx)
                    .label("Submit")
                    .primary()
                    .on_click(cx.listener(|_, _, cx| cx.emit(DismissEvent))),
            )
    }
}

pub struct PopupStory {
    focus_handle: FocusHandle,
    form: View<Form>,
    message: String,
    window_mode: bool,
}

impl PopupStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let form = Form::new(cx);
        Self {
            form,
            focus_handle: cx.focus_handle(),
            message: "".to_string(),
            window_mode: false,
        }
    }

    fn on_copy(&mut self, _: &Copy, _: &mut ViewContext<Self>) {
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

impl FocusableView for PopupStory {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopupStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let form = self.form.clone();
        let focus_handle = self.focus_handle.clone();
        let window_mode = self.window_mode;

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
            .context_menu({
                let focus_handle = focus_handle.clone();
                move |menu, _cx| {
                    menu.track_focus(focus_handle.clone())
                        .menu("Cut", Box::new(Cut))
                        .menu("Copy", Box::new(Copy))
                        .menu("Paste", Box::new(Paste))
                        .separator()
                        .menu("About", Box::new(SearchAll))
                }
            })
            .gap_6()
            .child(
                Switch::new("switch-window-mode")
                    .checked(self.window_mode)
                    .label("Use Window Popover")
                    .on_click(cx.listener(|this, checked, _| {
                        this.window_mode = *checked;
                    })),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_4().child(
                            Popover::new("info-top-left")
                                .when(self.window_mode, |this| this.window_mode())
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
                                                    .w(px(80.))
                                                    .small(),
                                            )
                                            .into_any()
                                    })
                                }),
                        ),
                    )
                    .child(
                        Popover::new("info-top-right")
                            .when(self.window_mode, |this| this.window_mode())
                            .anchor(AnchorCorner::TopRight)
                            .trigger(Button::new("info-top-right", cx).label("Top Right"))
                            .content(|cx| {
                                PopoverContent::new(cx, |cx| {
                                    v_flex()
                                        .gap_4()
                                        .w_96()
                                        .child("Hello, this is a Popover on the Top Right.")
                                        .child(Divider::horizontal())
                                        .child(
                                            Button::new("info1", cx)
                                                .label("Yes")
                                                .w(px(80.))
                                                .small(),
                                        )
                                        .into_any()
                                })
                            }),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        Popover::new("popup-menu")
                            .when(window_mode, |this| this.window_mode())
                            .trigger(Button::new("popup-menu-1", cx).icon(IconName::Ellipsis))
                            .content(move |cx| {
                                let focus_handle = focus_handle.clone();
                                PopupMenu::build(cx, |menu, _cx| {
                                    menu.track_focus(focus_handle)
                                        .menu("Copy", Box::new(Copy))
                                        .menu("Cut", Box::new(Cut))
                                        .menu("Paste", Box::new(Paste))
                                        .separator()
                                        .menu_with_icon(
                                            IconName::Search,
                                            "Search",
                                            Box::new(SearchAll),
                                        )
                                        .separator()
                                        .menu_with_check("Check Menu", true, Box::new(SearchAll))
                                })
                            }),
                    )
                    .child(self.message.clone()),
            )
            .child("Right click to open ContextMenu")
            .child(
                div().absolute().bottom_4().left_0().w_full().h_10().child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            Popover::new("info-bottom-left")
                                .when(self.window_mode, |this| this.window_mode())
                                .anchor(AnchorCorner::BottomLeft)
                                .trigger(
                                    Button::new("pop", cx).label("Popup with Form").w(px(300.)),
                                )
                                .content(move |_| form.clone()),
                        )
                        .child(
                            Popover::new("info-bottom-right")
                                .when(self.window_mode, |this| this.window_mode())
                                .anchor(AnchorCorner::BottomRight)
                                .mouse_button(MouseButton::Right)
                                .trigger(
                                    Button::new("pop", cx)
                                        .label("Mouse Right Click")
                                        .w(px(300.)),
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
                                                    .w(px(80.))
                                                    .small(),
                                            )
                                            .into_any()
                                    })
                                }),
                        ),
                ),
            )
    }
}
