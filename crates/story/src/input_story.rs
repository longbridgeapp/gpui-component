use gpui::{
    actions, AppContext, ClickEvent, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement as _, Render, Styled, View, ViewContext, VisualContext, WindowContext,
};

use ui::{button::Button, h_flex, input::TextInput, v_flex, Clickable, FocusableCycle, IconName};

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
}

impl InputStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    fn new(cx: &mut WindowContext) -> Self {
        let input1 = cx.new_view(|cx| {
            let mut input = TextInput::new(cx);
            input.set_text("Hello 世界", cx);
            input
        });

        let mask_input = cx.new_view(|cx| {
            let mut input = TextInput::new(cx);
            input.set_masked(true, cx);
            input.set_text("this-is-password", cx);
            input
        });

        let prefix_input1 = cx.new_view(|cx| {
            TextInput::new(cx)
                .prefix(IconName::Search.view(cx))
                .placeholder("Search some thing...")
        });
        let suffix_input1 = cx.new_view(|cx| {
            TextInput::new(cx)
                .suffix(IconName::Info.view(cx))
                .placeholder("Info here...")
        });
        let both_input1 = cx.new_view(|cx| {
            TextInput::new(cx)
                .prefix(IconName::Search.view(cx))
                .suffix(IconName::Info.view(cx))
                .placeholder("This input have prefix and suffix.")
        });

        Self {
            input1,
            input2: cx.new_view(|cx| {
                let mut input = TextInput::new(cx);
                input.set_placeholder("Enter text here...", cx);
                input
            }),
            mash_input: mask_input,
            disabled_input: cx.new_view(|cx| {
                let mut input = TextInput::new(cx);
                input.set_text("This is disabled input", cx);
                input.set_disabled(true, cx);
                input
            }),
            prefix_input1,
            suffix_input1,
            both_input1,
        }
    }

    #[allow(unused)]
    fn on_change(ev: &ClickEvent, cx: &mut WindowContext) {
        println!("Input changed: {:?}", ev);
    }

    fn tab(&mut self, _: &Tab, cx: &mut ViewContext<Self>) {
        self.cycle_focus(true, cx);
    }

    fn tab_prev(&mut self, _: &TabPrev, cx: &mut ViewContext<Self>) {
        self.cycle_focus(false, cx);
    }
}

impl FocusableCycle for InputStory {
    fn cycle_focus_handles(&self, cx: &mut ViewContext<Self>) -> Vec<FocusHandle> {
        [
            &self.input1,
            &self.input2,
            &self.disabled_input,
            &self.mash_input,
            &self.prefix_input1,
            &self.both_input1,
            &self.suffix_input1,
        ]
        .iter()
        .map(|v| v.focus_handle(cx))
        .collect()
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
                section("Normal Input", cx)
                    .child(self.input1.clone())
                    .child(self.input2.clone()),
            )
            .child(
                section("Input State", cx)
                    .child(self.disabled_input.clone())
                    .child(self.mash_input.clone()),
            )
            .child(
                section("Preifx and Suffix", cx)
                    .child(self.prefix_input1.clone())
                    .child(self.both_input1.clone())
                    .child(self.suffix_input1.clone()),
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
