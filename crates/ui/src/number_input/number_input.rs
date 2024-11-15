use super::modifier_handler::{ClickType, ScrollWheelDirection};
use super::traits::{SteppableNumber, ValidationPayload};
use crate::{
    button::Button,
    h_flex,
    input::{InputEvent, TextInput},
    Icon, IconName,
};
use bon::Builder;
use gpui::*;

const CONTEXT: &str = "NumberInput";

actions!(number_input, [Up, Down, BigUp, BigDown]);

pub fn init(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("up", Up, Some(CONTEXT)),
        KeyBinding::new("down", Down, Some(CONTEXT)),
        KeyBinding::new("ctrl-up", BigUp, Some(CONTEXT)),
        KeyBinding::new("ctrl-down", BigDown, Some(CONTEXT)),
    ])
}

#[derive(Builder, Clone)]
pub struct NumberInputConfig<T: SteppableNumber> {
    #[builder(default = T::zero())]
    initial_value: T,
    #[builder(default = T::one())]
    step: T,
    #[builder(default = T::one())]
    big_step: T,
    #[builder(default = T::min_value())]
    min: T,
    #[builder(default = T::max_value())]
    max: T,
    cleanable: Option<bool>,
}

pub enum Step {
    Normal,
    Big,
}

pub struct NumberInput<T: SteppableNumber> {
    value: T,
    is_focused: bool,
    focus_handle: FocusHandle,
    config: NumberInputConfig<T>,
    number_input: View<TextInput>,
}

impl<T: SteppableNumber> EventEmitter<InputEvent> for NumberInput<T> {}

impl<T: SteppableNumber> FocusableView for NumberInput<T> {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.number_input.focus_handle(cx)
    }
}

impl<T: SteppableNumber> NumberInput<T> {
    pub fn new(cx: &mut ViewContext<Self>, config: NumberInputConfig<T>) -> Self {
        let is_cleanable = &config.cleanable.unwrap_or(false);
        let initial_value = config.initial_value;

        let number_input = cx.new_view(|cx| {
            let mut input = if *is_cleanable {
                TextInput::new(cx)
                    .validate(|s| T::is_valid_number_string(s))
                    .cleanable()
            } else {
                TextInput::new(cx).validate(|s| T::is_valid_number_string(s))
            };
            input.set_text(initial_value.to_string(), cx);
            input
        });
        Self {
            value: initial_value,
            is_focused: false,
            focus_handle: cx.focus_handle(),
            config,
            number_input,
        }
    }

    fn set_number_text_repr(&mut self, number: T, cx: &mut ViewContext<Self>) {
        self.number_input.update(cx, |input, cx| {
            input.set_text(number.clone().to_string(), cx);
        });
        cx.notify();
    }

    fn increment(&mut self, step: Step, cx: &mut ViewContext<Self>) {
        let step = match step {
            Step::Normal => self.config.step,
            Step::Big => self.config.big_step,
        };

        match self.value.can_add(step) {
            ValidationPayload::Valid => {
                let new_value = self.value + step;
                if new_value > self.config.max {
                    self.value = self.config.max;
                } else {
                    self.value = new_value.clone();
                }
                self.set_number_text_repr(self.value, cx);
            }
            ValidationPayload::Invalid { remainder: _ } => {
                self.value = self.config.max;
                self.set_number_text_repr(self.value, cx);
            }
        }
    }

    fn decrement(&mut self, step: Step, cx: &mut ViewContext<Self>) {
        let step = match step {
            Step::Normal => self.config.step,
            Step::Big => self.config.big_step,
        };

        match self.value.can_subtract(step) {
            ValidationPayload::Valid => {
                let new_value = self.value - step;
                if new_value < self.config.min {
                    self.value = self.config.min;
                } else {
                    self.value = new_value.clone();
                }
                self.set_number_text_repr(self.value, cx);
            }
            ValidationPayload::Invalid { remainder: _ } => {
                self.value = self.config.min;
                self.set_number_text_repr(self.value, cx);
            }
        }
    }

    fn focus_number_input(&mut self, cx: &mut ViewContext<Self>) {
        // this is probably unnecessary
        // since clicking on the button will bring focus to that button
        if self.number_input.focus_handle(cx).is_focused(cx) {
            return;
        }
        self.number_input.focus_handle(cx).focus(cx);
    }

    fn click_increment(&mut self, event: &ClickEvent, cx: &mut ViewContext<Self>) {
        if let Some(click_type) = ClickType::from_event(event) {
            match click_type {
                ClickType::Normal => self.increment(Step::Normal, cx),
                ClickType::Big => self.increment(Step::Big, cx),
            }
            self.focus_number_input(cx);
        }
    }

    fn click_decrement(&mut self, event: &ClickEvent, cx: &mut ViewContext<Self>) {
        if let Some(click_type) = ClickType::from_event(event) {
            match click_type {
                ClickType::Normal => self.decrement(Step::Normal, cx),
                ClickType::Big => self.decrement(Step::Big, cx),
            }
            self.focus_number_input(cx);
        }
    }

    fn keybind_up(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        self.increment(Step::Normal, cx);
    }

    fn keybind_big_up(&mut self, _: &BigUp, cx: &mut ViewContext<Self>) {
        self.increment(Step::Big, cx);
    }

    fn keybind_down(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
        self.decrement(Step::Normal, cx);
    }

    fn keybind_big_down(&mut self, _: &BigDown, cx: &mut ViewContext<Self>) {
        self.decrement(Step::Big, cx);
    }

    fn on_number_input_event(
        &mut self,
        _input: View<TextInput>,
        event: &InputEvent,
        _cx: &mut ViewContext<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                let number = T::from_string(text);
                if let Some(number) = number {
                    self.value = number;
                }
            }
            InputEvent::Focus => self.is_focused = true,
            InputEvent::Blur => self.is_focused = false,
            InputEvent::Cleaned => {
                self.value = self.config.initial_value;
                self.set_number_text_repr(self.config.initial_value, _cx);
            }
            _ => {}
        };
    }
}

impl<T: SteppableNumber> Render for NumberInput<T> {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let number_input = self.number_input.to_owned();

        cx.subscribe(&number_input, Self::on_number_input_event)
            .detach();

        h_flex()
            .key_context(CONTEXT)
            .id("number-input-container")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::keybind_up))
            .on_action(cx.listener(Self::keybind_down))
            .on_action(cx.listener(Self::keybind_big_up))
            .on_action(cx.listener(Self::keybind_big_down))
            .w_full()
            .gap_1()
            .child(number_input)
            .on_scroll_wheel(cx.listener(move |this, event, cx| {
                if !*&this.is_focused {
                    return;
                }
                if let Some(direction) = ScrollWheelDirection::from_event(event) {
                    match direction {
                        ScrollWheelDirection::Up => this.increment(Step::Normal, cx),
                        ScrollWheelDirection::Down => this.decrement(Step::Normal, cx),
                        ScrollWheelDirection::BigUp => this.increment(Step::Big, cx),
                        ScrollWheelDirection::BigDown => this.decrement(Step::Big, cx),
                    }
                    cx.notify();
                }
            }))
            .child(
                h_flex()
                    .id("number-input-controls")
                    .gap_0p5()
                    .child(
                        Button::new("increment")
                            .icon(Icon::new(IconName::ChevronUp))
                            .on_click(cx.listener(Self::click_increment)),
                    )
                    .child(
                        Button::new("decrement")
                            .icon(Icon::new(IconName::ChevronDown))
                            .on_click(cx.listener(Self::click_decrement)),
                    ),
            )
    }
}
