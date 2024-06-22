use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, IntoElement, ParentElement as _, Render,
    Styled as _, ViewContext, VisualContext, WindowContext,
};

use crate::text_field::TextField;

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
                    TextField::new(cx, "Enter text here...", false)
                })
                .child({
                        let input = TextField::new(cx, "Enter text here...", false);
                        input
                }),
        )
    }
}
