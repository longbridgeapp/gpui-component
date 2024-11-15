use gpui::{
    actions, AppContext, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement as _, Render, Styled, View, ViewContext, VisualContext, WindowContext,
};

use crate::section;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use ui::{
    h_flex,
    input::InputEvent,
    number_input::{NumberInput, NumberInputConfig, SteppableNumber},
    scroll::ScrollbarAxis,
    v_flex, FocusableCycle, StyledExt,
};
actions!(input_story, [Tab, TabPrev]);

const CONTEXT: &str = "NumberInputStory";

pub fn init(cx: &mut AppContext) {
    ui::number_input::init(cx);
    cx.bind_keys([
        KeyBinding::new("shift-tab", TabPrev, Some(CONTEXT)),
        KeyBinding::new("tab", Tab, Some(CONTEXT)),
    ])
}

pub struct NumberInputStory {
    rust_decimal_ni: View<NumberInput<Decimal>>,
    f32_ni: View<NumberInput<f32>>,
    i8_ni: View<NumberInput<i8>>,
    u8_ni: View<NumberInput<u8>>,
}

impl super::Story for NumberInputStory {
    fn title() -> &'static str {
        "NumberInput"
    }

    fn description() -> &'static str {
        "Generic number input with step and big step (Ctrl modifier).\nSupports Arrow and scroll wheel inputs."
    }

    fn closeable() -> bool {
        false
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }
}

impl NumberInputStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let rust_decimal_ni = cx.new_view(|cx| {
            let input = NumberInput::<Decimal>::new(
                cx,
                NumberInputConfig::builder()
                    .cleanable(true)
                    .step(dec!(3.33))
                    .big_step(rust_decimal::Decimal::PI)
                    .initial_value(dec!(3.33))
                    .build(),
            );
            input
        });

        cx.subscribe(&rust_decimal_ni, Self::on_input_event)
            .detach();

        let f32_ni = cx.new_view(|cx| {
            let input = NumberInput::<f32>::new(
                cx,
                NumberInputConfig::builder()
                    .cleanable(true)
                    .step(4.52)
                    .big_step(8.96)
                    .initial_value(43.22)
                    .build(),
            );
            input
        });

        cx.subscribe(&f32_ni, Self::on_input_event).detach();

        let i8_ni = cx.new_view(|cx| {
            let input = NumberInput::<i8>::new(
                cx,
                NumberInputConfig::builder()
                    .cleanable(true)
                    .big_step(10)
                    .initial_value(-100)
                    .build(),
            );
            input
        });

        cx.subscribe(&i8_ni, Self::on_input_event).detach();
        let u8_ni = cx.new_view(|cx| {
            let input = NumberInput::<u8>::new(
                cx,
                NumberInputConfig::builder()
                    .cleanable(true)
                    .big_step(10)
                    .initial_value(100)
                    .build(),
            );
            input
        });

        cx.subscribe(&u8_ni, Self::on_input_event).detach();

        Self {
            rust_decimal_ni,
            f32_ni,
            i8_ni,
            u8_ni,
        }
    }

    fn tab(&mut self, _: &Tab, cx: &mut ViewContext<Self>) {
        self.cycle_focus(true, cx);
    }

    fn tab_prev(&mut self, _: &TabPrev, cx: &mut ViewContext<Self>) {
        self.cycle_focus(false, cx);
    }

    fn on_input_event<T: SteppableNumber>(
        &mut self,
        _: View<NumberInput<T>>,
        event: &InputEvent,
        _cx: &mut ViewContext<Self>,
    ) {
        match event {
            InputEvent::Cleaned => println!("Cleaned"),
            InputEvent::Change(text) => println!("Change: {}", text),
            InputEvent::PressEnter => println!("PressEnter"),
            InputEvent::Focus => println!("Focus"),
            InputEvent::Blur => println!("Blur"),
        };
    }
}

impl FocusableCycle for NumberInputStory {
    fn cycle_focus_handles(&self, cx: &mut ViewContext<Self>) -> Vec<FocusHandle> {
        [
            self.rust_decimal_ni.focus_handle(cx),
            self.f32_ni.focus_handle(cx),
            self.u8_ni.focus_handle(cx),
            self.i8_ni.focus_handle(cx),
        ]
        .to_vec()
    }
}
impl gpui::FocusableView for NumberInputStory {
    fn focus_handle(&self, cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.rust_decimal_ni.focus_handle(cx)
    }
}

impl Render for NumberInputStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("input-story")
            .scrollable(cx.entity_id(), ScrollbarAxis::Vertical)
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
                    .child(section("rust_decimal::Decimal", cx).child(self.rust_decimal_ni.clone()))
                    .child(section("Float", cx).child(self.f32_ni.clone())),
            )
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(section("Unsigned Int", cx).child(self.u8_ni.clone()))
                    .child(section("Signed Int", cx).child(self.i8_ni.clone())),
            )
    }
}
