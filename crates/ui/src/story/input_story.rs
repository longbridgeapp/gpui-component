use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, IntoElement, ParentElement as _, Render,
    Styled as _, ViewContext, VisualContext, WindowContext,
};

use crate::input::Input;

use super::story_case;

pub struct InputStory;

impl InputStory {
    fn on_change(ev: &ClickEvent, cx: &mut WindowContext) {
        println!("Input changed: {:?}", ev);
    }
}

impl Render for InputStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case("Input", "A text input field.").child(
            div()
                .flex()
                .flex_col()
                .justify_start()
                .gap_3()
                .child({
                    cx.new_view(|cx| {
                        let input = Input::new("input1", cx);
                        input.set_placeholder("Enter text here...", cx);
                        input
                    })
                })
                .child({
                    cx.new_view(|cx| {
                        let input = Input::new("input1", cx);
                        input.set_placeholder("Enter text here...", cx);
                        input.set_text("Hello, world!", cx);
                        input
                    })
                }),
        )
    }
}
