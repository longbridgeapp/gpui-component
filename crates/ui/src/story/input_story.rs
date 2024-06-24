use gpui::{
    div, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, ViewContext,
    VisualContext as _, WindowContext,
};

use crate::text_field::TextField;

use super::story_case;

pub struct InputStory;

impl InputStory {
    #[allow(unused)]
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
                .child(cx.new_view(|cx| TextField::new("Enter text here...", false, cx)))
                .child(cx.new_view(|cx| TextField::new("Enter text here...", false, cx))),
        )
    }
}
