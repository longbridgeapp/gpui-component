use gpui::{
    div, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext, WindowContext,
};

use crate::text_field::TextField;

use super::story_case;

pub struct InputStory {
    input1: View<TextField>,
    input2: View<TextField>,
    mash_input: View<TextField>,
    disabled_input: View<TextField>,
}

impl InputStory {
    pub(crate) fn new(cx: &mut WindowContext) -> Self {
        let input1 = cx.new_view(|cx| TextField::new(cx).set_text("Hello 世界", cx));

        let mask_input = cx.new_view(|cx| {
            TextField::new(cx)
                .set_masked(true, cx)
                .set_text("this-is-password", cx)
        });

        Self {
            input1,
            input2: cx.new_view(|cx| TextField::new(cx).set_placeholder("Enter text here...", cx)),
            mash_input: mask_input,
            disabled_input: cx.new_view(|cx| {
                TextField::new(cx)
                    .set_text("This is disabled input", cx)
                    .set_disabled(true, cx)
            }),
        }
    }

    #[allow(unused)]
    fn on_change(ev: &ClickEvent, cx: &mut WindowContext) {
        println!("Input changed: {:?}", ev);
    }
}

impl Render for InputStory {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case("Input", "A text input field.").child(
            div()
                .flex()
                .flex_col()
                .justify_start()
                .gap_3()
                .child(self.input1.clone())
                .child(self.input2.clone())
                .child(self.disabled_input.clone())
                .child(self.mash_input.clone()),
        )
    }
}
