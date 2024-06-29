use gpui::{div, px, Element, IntoElement, ParentElement as _, Render, Styled as _, ViewContext};
use ui::{
    button::{Button, ButtonSize},
    divider::Divider,
    h_flex,
    popover::{Popover, PopoverContent},
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
    fn render(&mut self, _: &mut ViewContext<Self>) -> impl IntoElement {
        story_case(
            "Popover",
            "Displays rich content in a portal, triggered by a button.",
        )
        .child(
            h_flex().items_center().justify_between().child(
                Popover::new("info-top-left")
                    .trigger(Button::new("info", "Top Left").width(px(300.)))
                    .content(|cx| {
                        PopoverContent::new(|_| {
                                v_flex().gap_4().child("Hello, this is a Popover.")
                                    .child(Divider::horizontal())
                                    .child(
                                        Button::new("info1", "Yes")
                                            .width(px(80.))
                                            .size(ButtonSize::Small)
                                    ).into_any()
                            },
                            cx,
                        )
                    }),
            ).child(
                Popover::new("info-top-right")
                .trigger(Button::new("info", "Top Right").width(px(300.)))
                .content(|cx| {
                    PopoverContent::new(|_|
                        "Hello, this is a Popover.\nYou can click outside to dissmis.".into_any(),
                        cx,
                    )
                }),
            )
        )
        .child(
            div().absolute().bottom_4().left_0().w_full().h_10().child(
                h_flex().items_center().justify_between().child(
                Popover::new("info-bottom-left")
                    .trigger(Button::new("pop", "Bottom Left").width(px(300.)))
                    .content(|cx| {
                        PopoverContent::new(|_| "这是另外一个 Popover。\n你可以点击外部来关闭。\nThis popover has position bottom_4().left_0().w_full().h_10().".into_any(), cx)
                    })).child(
                Popover::new("info-bottom-right")
                    .trigger(Button::new("pop", "Bottom Right").width(px(300.)))
                    .content(|cx| {
                        PopoverContent::new(|_| "这是另外一个 Popover。\n你可以点击外部来关闭。\nThis popover has position bottom_4().left_0().w_full().h_10().".into_any(), cx)
                    }))


            ),
        )
    }
}
