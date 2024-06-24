use gpui::{
    div, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, ViewContext,
    WindowContext,
};

use crate::text_field::TextField;

use super::story_case;

pub struct InputStory {
    input1: TextField,
    input2: TextField,
}

impl InputStory {
    pub(crate) fn new(cx: &mut ViewContext<Self>) -> Self {
        let input1 = TextField::new("Enter text here...", false, cx);
        input1
            .view
            .update(cx, |text_view, cx| text_view.set_text("Hello 世界", cx));

        Self {
            input1,
            input2: TextField::new("Enter text here...", true, cx),
        }
    }

    #[allow(unused)]
    fn on_change(ev: &ClickEvent, cx: &mut WindowContext) {
        println!("Input changed: {:?}", ev);
    }
}

impl Render for InputStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let input1 = self.input1.clone();
        let input2 = self.input2.clone();

        story_case("Input", "A text input field.").child(
            div()
                .flex()
                .flex_col()
                .justify_start()
                .gap_3()
                .child(input1)
                .child(input2),
        )
    }
}
