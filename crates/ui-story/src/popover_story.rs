use gpui::{
    div, px, Entity, IntoElement, ParentElement as _, Render, Styled as _, ViewContext,
    VisualContext,
};
use ui::{
    button::Button,
    popover::{Popover, PopoverContent},
    v_flex,
};

pub struct PopoverStory {
    // info_btn: View<Popover>,
}

impl PopoverStory {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            // info_btn: cx.new_view(|cx| {}),
        }
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
