use gpui::{
    div, px, Element, IntoElement, ParentElement as _, Render, Styled as _, ViewContext,
    VisualContext,
};
use ui::{
    button::{Button, ButtonSize},
    divider::Divider,
    h_flex,
    popover::Popover,
    v_flex,
};

use crate::story_case;

pub struct PopoverStory {}

impl PopoverStory {
    pub fn new(_: &mut ViewContext<Self>) -> Self {
        Self {}
    }
}

impl Render for PopoverStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case(
            "Popover",
            "Displays rich content in a portal, triggered by a button.",
        )
        .child(
            h_flex().items_center().justify_between().child(
                v_flex().gap_4().child(
                    Popover::new("info-top-left", cx)
                        .trigger(Button::new("info-top-left", "Top Left"))
                        .content(
                            |_| {
                                v_flex()
                                    .gap_4()
                                    .child("Hello, this is a Popover.")
                                    .child(Divider::horizontal())
                                    .child(
                                        Button::new("info1", "Yes")
                                            .width(px(80.))
                                            .size(ButtonSize::Small),
                                    )
                                    .into_any()
                            },
                            cx,
                        ),
                ),
            ),
        )
        // .child(
        //     div().absolute().bottom_4().left_0().w_full().h_10().child(
        //         h_flex().items_center().justify_between().child(
        //         Popover::new("info-bottom-left")
        //             .trigger(Button::new("pop", "Bottom Left").width(px(300.)).into_any_element())
        //             .content(|cx|
        //                  "这是另外一个 Popover。\n你可以点击外部来关闭。\nThis popover has position bottom_4().left_0().w_full().h_10().".into_any()
        //             )).child(
        //         Popover::new("info-bottom-right")
        //             .trigger(Button::new("pop", "Bottom Right").width(px(300.)))
        //             .content(|cx| {
        //                 PopoverContent::new(|_| "这是另外一个 Popover。\n你可以点击外部来关闭。\nThis popover has position bottom_4().left_0().w_full().h_10().".into_any(), cx)
        //             }))

        //     ),
        // )
    }
}
