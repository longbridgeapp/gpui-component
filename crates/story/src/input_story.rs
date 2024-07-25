use gpui::{
    actions, div, AppContext, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement as _, Render, SharedString, Styled, View, ViewContext, VisualContext,
    WindowContext,
};

use ui::{
    button::Button,
    h_flex,
    input::{InputEvent, InputOtp, TextInput},
    prelude::FluentBuilder as _,
    v_flex, Clickable, FocusableCycle, IconName, Size,
};

use crate::section;

actions!(input_story, [Tab, TabPrev]);

pub fn init(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("shift-tab", TabPrev, Some("InputStory")),
        KeyBinding::new("tab", Tab, Some("InputStory")),
    ])
}

pub struct InputStory {
    input1: View<TextInput>,
    input2: View<TextInput>,
    mash_input: View<TextInput>,
    disabled_input: View<TextInput>,
    prefix_input1: View<TextInput>,
    suffix_input1: View<TextInput>,
    both_input1: View<TextInput>,
    large_input: View<TextInput>,
    small_input: View<TextInput>,
    otp_input: View<InputOtp>,
    otp_value: Option<SharedString>,
    opt_input2: View<InputOtp>,
}

impl InputStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let input1 = cx.new_view(|cx| {
            let mut input = TextInput::new(cx).cleanable(true);
            input.set_text(
                "Hello 世界，this is GPUI component, this is a long text.",
                cx,
            );
            input
        });

        cx.subscribe(&input1, Self::on_input_event).detach();

        let input2 = cx.new_view(|cx| TextInput::new(cx).placeholder("Enter text here..."));

        cx.subscribe(&input2, Self::on_input_event).detach();

        let mask_input = cx.new_view(|cx| {
            let mut input = TextInput::new(cx).cleanable(true);
            input.set_masked(true, cx);
            input.set_text("this-is-password", cx);
            input
        });

        let prefix_input1 = cx.new_view(|cx| {
            TextInput::new(cx)
                .prefix(|_| div().child(IconName::Search).ml_3())
                .placeholder("Search some thing...")
                .cleanable(true)
        });
        let suffix_input1 = cx.new_view(|cx| {
            TextInput::new(cx)
                .suffix(|_| div().child(IconName::Info).mr_3())
                .placeholder("This input only support [a-zA-Z0-9] characters.")
                .pattern(regex::Regex::new(r"^[a-zA-Z0-9]*$").unwrap())
                .cleanable(true)
        });
        let both_input1 = cx.new_view(|cx| {
            TextInput::new(cx)
                .prefix(|_| div().child(IconName::Search).ml_3())
                .suffix(|_| div().child(IconName::Info).mr_3())
                .cleanable(true)
                .placeholder("This input have prefix and suffix.")
        });

        let otp_input = cx.new_view(|cx| InputOtp::new(6, cx).masked(true));
        cx.subscribe(&otp_input, |this, _, ev: &InputEvent, cx| match ev {
            InputEvent::Change(text) => {
                this.otp_value = Some(text.clone());
                cx.notify();
            }
            _ => {}
        })
        .detach();

        Self {
            input1,
            input2,
            mash_input: mask_input,
            disabled_input: cx.new_view(|cx| {
                let mut input = TextInput::new(cx);
                input.set_text("This is disabled input", cx);
                input.set_disabled(true, cx);
                input
            }),
            large_input: cx.new_view(|cx| {
                TextInput::new(cx)
                    .size(Size::Large)
                    .placeholder("Large input")
            }),
            small_input: cx.new_view(|cx| {
                TextInput::new(cx)
                    .size(Size::Small)
                    .validate(|s| s.parse::<f32>().is_ok())
                    .placeholder("validate to limit float number.")
            }),
            prefix_input1,
            suffix_input1,
            both_input1,
            otp_input,
            otp_value: None,
            opt_input2: cx.new_view(|cx| InputOtp::new(6, cx).groups(3)),
        }
    }

    fn tab(&mut self, _: &Tab, cx: &mut ViewContext<Self>) {
        self.cycle_focus(true, cx);
    }

    fn tab_prev(&mut self, _: &TabPrev, cx: &mut ViewContext<Self>) {
        self.cycle_focus(false, cx);
    }

    fn on_input_event(
        &mut self,
        _: View<TextInput>,
        event: &InputEvent,
        _cx: &mut ViewContext<Self>,
    ) {
        match event {
            InputEvent::Change(text) => println!("Change: {}", text),
            InputEvent::PressEnter => println!("PressEnter"),
            InputEvent::Focus => println!("Focus"),
            InputEvent::Blur => println!("Blur"),
        };
    }
}

impl FocusableCycle for InputStory {
    fn cycle_focus_handles(&self, cx: &mut ViewContext<Self>) -> Vec<FocusHandle> {
        [
            self.input1.focus_handle(cx),
            self.input2.focus_handle(cx),
            self.disabled_input.focus_handle(cx),
            self.mash_input.focus_handle(cx),
            self.prefix_input1.focus_handle(cx),
            self.both_input1.focus_handle(cx),
            self.suffix_input1.focus_handle(cx),
            self.large_input.focus_handle(cx),
            self.small_input.focus_handle(cx),
            self.otp_input.focus_handle(cx),
        ]
        .to_vec()
    }
}

impl Render for InputStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .key_context("InputStory")
            .on_action(cx.listener(Self::tab))
            .on_action(cx.listener(Self::tab_prev))
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(
                        section("Normal Input", cx)
                            .child(self.input1.clone())
                            .child(self.input2.clone()),
                    )
                    .child(
                        section("Input State", cx)
                            .child(self.disabled_input.clone())
                            .child(self.mash_input.clone()),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(
                        section("Prefix and Suffix", cx)
                            .child(self.prefix_input1.clone())
                            .child(self.both_input1.clone())
                            .child(self.suffix_input1.clone()),
                    )
                    .child(
                        section("Input Size", cx)
                            .child(self.large_input.clone())
                            .child(self.small_input.clone()),
                    ),
            )
            .child(
                section("Input OTP", cx).child(
                    v_flex()
                        .gap_3()
                        .child(self.otp_input.clone())
                        .when_some(self.otp_value.clone(), |this, otp| {
                            this.child(format!("You input OTP: {}", otp))
                        })
                        .child(self.opt_input2.clone()),
                ),
            )
            .child(
                h_flex()
                    .items_center()
                    .w_full()
                    .gap_3()
                    .child(
                        Button::new("btn-submit", cx)
                            .w_full()
                            .style(ui::button::ButtonStyle::Primary)
                            .label("Submit")
                            .on_click(cx.listener(|_, _, cx| cx.dispatch_action(Box::new(Tab)))),
                    )
                    .child(
                        Button::new("btn-cancel", cx)
                            .w_full()
                            .label("Cancel")
                            .into_element(),
                    ),
            )
    }
}
