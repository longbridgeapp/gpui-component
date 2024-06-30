use gpui::{
    px, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};

use ui::{
    button::{Button, ButtonSize, ButtonStyle},
    h_flex, v_flex, Clickable, Disableable as _, Icon, IconName, Selectable,
};

use super::story_case;

pub struct ButtonStory {}

impl ButtonStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self {})
    }

    fn on_click(ev: &ClickEvent, _: &mut WindowContext) {
        println!("Button clicked! {:?}", ev);
    }
}

impl Render for ButtonStory {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .justify_start()
            .debug()
            .gap_6()
            .child(
                v_flex()
                    .w(px(360.))
                    .gap_6()
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
                    ),
            )
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        Button::new("button-icon-1", "Confirm")
                            .icon(IconName::Check)
                            .style(ButtonStyle::Primary)
                            .on_click(Self::on_click),
                    )
                    .child(
                        Button::new("button-icon-2", "Abort")
                            .icon(IconName::Close)
                            .style(ButtonStyle::Secondary)
                            .on_click(Self::on_click),
                    )
                    .child(
                        Button::new("button-icon-3", "Maximize")
                            .icon(Icon::new(IconName::Maximize))
                            .style(ButtonStyle::Secondary)
                            .on_click(Self::on_click),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_6()
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
                h_flex()
                    .items_center()
                    .gap_6()
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
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_6()
                    .child(
                        Button::new("button-6", "Selected Button")
                            .style(ButtonStyle::Primary)
                            .selected(true),
                    )
                    .child(
                        Button::new("button-7", "Selected Button")
                            .style(ButtonStyle::Secondary)
                            .selected(true),
                    )
                    .child(
                        Button::new("button-8", "Selected Button")
                            .style(ButtonStyle::Danger)
                            .selected(true),
                    ),
            )
    }
}
