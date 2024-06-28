use gpui::{px, IntoElement, ParentElement as _, Render, Styled as _, ViewContext};
use ui::{
    button::Button,
    popover::{Popover, PopoverContent},
    v_flex,
};

pub struct PopoverStory {}

impl PopoverStory {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {}
    }
}

impl Render for PopoverStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex().gap_6().child(
            Popover::new("info")
                .trigger(Button::new("info", "Show Popover").width(px(300.)))
                .content(|cx| {
                    PopoverContent::new(
                        "Hello, this is a Popover.\nYou can click outside to dissmis.",
                        cx,
                    )
                }),
        )
    }
}
