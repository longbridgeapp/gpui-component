use gpui::{
    ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};

use ui::{
    button::{Button, ButtonSize, ButtonStyle},
    h_flex, v_flex, Clickable, Disableable as _, Icon, IconName, Selectable,
};

use crate::section;

pub struct ButtonStory {}

impl ButtonStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self {})
    }

    fn on_click(ev: &ClickEvent, _: &mut WindowContext) {
        println!("Button clicked! {:?}", ev);
    }
}

impl Render for ButtonStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .gap_6()
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        section("Normal Button", cx)
                            .child(
                                Button::new("button-1", cx)
                                    .label("Primary Button")
                                    .style(ButtonStyle::Primary)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-2", cx)
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-4", cx)
                                    .label("Danger Button")
                                    .style(ButtonStyle::Danger)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-5", cx)
                                    .label("Outline Button")
                                    .style(ButtonStyle::Outline)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-5-ghost", cx)
                                    .label("Ghost Button")
                                    .style(ButtonStyle::Ghost)
                                    .on_click(Self::on_click),
                            ),
                    )
                    .child(
                        section("Button with Icon", cx)
                            .child(
                                Button::new("button-icon-1", cx)
                                    .label("Confirm")
                                    .icon(IconName::Check)
                                    .style(ButtonStyle::Primary)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-2", cx)
                                    .label("Abort")
                                    .icon(IconName::Close)
                                    .style(ButtonStyle::Secondary)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-icon-3", cx)
                                    .label("Maximize")
                                    .icon(Icon::new(IconName::Maximize))
                                    .style(ButtonStyle::Secondary)
                                    .on_click(Self::on_click),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        section("Small Size", cx)
                            .child(
                                Button::new("button-6", cx)
                                    .label("Primary Button")
                                    .style(ButtonStyle::Primary)
                                    .size(ButtonSize::Small)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-7", cx)
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .size(ButtonSize::Small)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8", cx)
                                    .label("Danger Button")
                                    .style(ButtonStyle::Danger)
                                    .size(ButtonSize::Small)
                                    .on_click(Self::on_click),
                            ),
                    )
                    .child(
                        section("XSmall Size", cx)
                            .child(
                                Button::new("button-xs-1", cx)
                                    .label("Primary Button")
                                    .style(ButtonStyle::Primary)
                                    .size(ButtonSize::XSmall)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-2", cx)
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .size(ButtonSize::XSmall)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3", cx)
                                    .label("Danger Button")
                                    .style(ButtonStyle::Danger)
                                    .size(ButtonSize::XSmall)
                                    .on_click(Self::on_click),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        section("Disabled Button", cx)
                            .child(
                                Button::new("button-disabled1", cx)
                                    .label("Disabled Button")
                                    .style(ButtonStyle::Primary)
                                    .on_click(Self::on_click)
                                    .disabled(true),
                            )
                            .child(
                                Button::new("button-disabled1", cx)
                                    .label("Disabled Button")
                                    .style(ButtonStyle::Secondary)
                                    .on_click(Self::on_click)
                                    .disabled(true),
                            )
                            .child(
                                Button::new("button-disabled1", cx)
                                    .label("Disabled Button")
                                    .style(ButtonStyle::Danger)
                                    .on_click(Self::on_click)
                                    .disabled(true),
                            ),
                    )
                    .child(
                        section("Selected Style", cx)
                            .child(
                                Button::new("button-selected-1", cx)
                                    .label("Selected Button")
                                    .style(ButtonStyle::Primary)
                                    .selected(true),
                            )
                            .child(
                                Button::new("button-selected-2", cx)
                                    .label("Selected Button")
                                    .style(ButtonStyle::Secondary)
                                    .selected(true),
                            )
                            .child(
                                Button::new("button-selected-3", cx)
                                    .label("Selected Button")
                                    .style(ButtonStyle::Danger)
                                    .selected(true),
                            ),
                    ),
            )
            .child(
                section("Icon Button", cx)
                    .child(
                        Button::new("icon-button-0", cx)
                            .icon(IconName::Search)
                            .style(ButtonStyle::Primary),
                    )
                    .child(Button::new("icon-button-1", cx).icon(IconName::Info))
                    .child(
                        Button::new("icon-button-2", cx)
                            .icon(IconName::Close)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-3", cx)
                            .icon(IconName::Search)
                            .size(ButtonSize::Small)
                            .style(ButtonStyle::Primary),
                    )
                    .child(
                        Button::new("icon-button-0-outline", cx)
                            .icon(IconName::Search)
                            .style(ButtonStyle::Outline),
                    )
                    .child(
                        Button::new("icon-button-1", cx)
                            .icon(IconName::Info)
                            .style(ButtonStyle::Ghost),
                    ),
            )
            .child(
                section("Icon Button", cx)
                    .child(
                        Button::new("icon-button-4", cx)
                            .icon(IconName::Info)
                            .size(ButtonSize::Small),
                    )
                    .child(
                        Button::new("icon-button-5", cx)
                            .icon(IconName::Close)
                            .size(ButtonSize::Small)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-6", cx)
                            .icon(IconName::Search)
                            .size(ButtonSize::XSmall)
                            .style(ButtonStyle::Primary),
                    )
                    .child(
                        Button::new("icon-button-7", cx)
                            .icon(IconName::Info)
                            .size(ButtonSize::XSmall),
                    )
                    .child(
                        Button::new("icon-button-8", cx)
                            .icon(IconName::Close)
                            .size(ButtonSize::XSmall)
                            .style(ButtonStyle::Danger),
                    ),
            )
    }
}
