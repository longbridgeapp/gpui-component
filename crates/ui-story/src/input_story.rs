use gpui::{
    ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext, WindowContext,
};

use ui::{input::TextInput, v_flex};

pub struct InputStory {
    input1: View<TextInput>,
    input2: View<TextInput>,
    mash_input: View<TextInput>,
    disabled_input: View<TextInput>,
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
        }
    }

    #[allow(unused)]
    fn on_change(ev: &ClickEvent, cx: &mut WindowContext) {
        println!("Input changed: {:?}", ev);
    }
}

impl Render for InputStory {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .justify_start()
            .gap_3()
            .child(self.input1.clone())
            .child(self.input2.clone())
            .child(self.disabled_input.clone())
            .child(self.mash_input.clone())
    }
}
