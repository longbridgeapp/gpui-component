use gpui::{
    div, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, ViewContext,
    WindowContext,
};

use crate::{
    button::{Button, ButtonSize, ButtonStyle},
    disableable::{Clickable, Disableable as _},
};

use super::story_case;

pub struct ButtonStory;

impl ButtonStory {
    fn on_click(ev: &ClickEvent, cx: &mut WindowContext) {
        println!("Button clicked! {:?}", ev);
    }
}

impl Render for ButtonStory {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case(
            "Button",
            "Displays a button or a component that looks like a button.",
        )
        .child(
            div()
                .flex()
                .flex_col()
                .justify_start()
                .gap_3()
                .child(
                    Button::new("button-1", "Primary Button")
                        .style(ButtonStyle::Primary)
                        .on_click(Self::on_click),
                )
                .child(
                    Button::new("button-2", "Secondary Button")
                        .style(ButtonStyle::Secondary)
                        .on_click(Self::on_click),
                )
                .child(
                    Button::new("button-4", "Danger Button")
                        .style(ButtonStyle::Danger)
                        .on_click(Self::on_click),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            Button::new("button-disabled1", "Disabled Button")
                                .style(ButtonStyle::Primary)
                                .on_click(Self::on_click)
                                .disabled(true),
                        )
                        .child(
                            Button::new("button-disabled1", "Disabled Button")
                                .style(ButtonStyle::Secondary)
                                .on_click(Self::on_click)
                                .disabled(true),
                        )
                        .child(
                            Button::new("button-disabled1", "Disabled Button")
                                .style(ButtonStyle::Danger)
                                .on_click(Self::on_click)
                                .disabled(true),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            Button::new("button-6", "Primary Button")
                                .style(ButtonStyle::Primary)
                                .size(ButtonSize::Small)
                                .on_click(Self::on_click),
                        )
                        .child(
                            Button::new("button-7", "Secondary Button")
                                .style(ButtonStyle::Secondary)
                                .size(ButtonSize::Small)
                                .on_click(Self::on_click),
                        )
                        .child(
                            Button::new("button-8", "Danger Button")
                                .style(ButtonStyle::Danger)
                                .size(ButtonSize::Small)
                                .on_click(Self::on_click),
                        ),
                ),
        )
    }
}
