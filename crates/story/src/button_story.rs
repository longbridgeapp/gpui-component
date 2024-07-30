use gpui::{
    px, ClickEvent, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};

use ui::{
    button::{Button, ButtonCustomStyle, ButtonStyle},
    h_flex,
    theme::ActiveTheme,
    v_flex, Clickable, Disableable as _, Icon, IconName, Selectable, Size,
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
                            )
                            .child(
                                Button::new("button-6-custom", cx)
                                    .label("Custom Button")
                                    .style(ButtonStyle::Custom(
                                        ButtonCustomStyle::new(cx)
                                            .color(cx.theme().muted)
                                            .foreground(cx.theme().destructive)
                                            .border(cx.theme().scrollbar)
                                            .hover(cx.theme().tab_active_foreground)
                                            .active(cx.theme().selection),
                                    ))
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
                            )
                            .child(
                                Button::new("button-icon-4", cx)
                                    .style(ButtonStyle::Secondary)
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child("Custom Child")
                                            .child(IconName::ChevronDown)
                                            .child(IconName::Eye),
                                    )
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
                                    .size(Size::Small)
                                    .loading(true)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-7", cx)
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .size(Size::Small)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8", cx)
                                    .label("Danger Button")
                                    .style(ButtonStyle::Danger)
                                    .size(Size::Small)
                                    .on_click(Self::on_click),
                            ),
                    )
                    .child(
                        section("XSmall Size", cx)
                            .child(
                                Button::new("button-xs-1", cx)
                                    .label("Primary Button")
                                    .style(ButtonStyle::Primary)
                                    .size(Size::XSmall)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-2", cx)
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .size(Size::XSmall)
                                    .loading(true)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3", cx)
                                    .label("Danger Button")
                                    .style(ButtonStyle::Danger)
                                    .size(Size::XSmall)
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
                                    .disabled(true)
                                    .loading(true),
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
                    .child(
                        Button::new("icon-button-1", cx)
                            .icon(IconName::Info)
                            .loading(true),
                    )
                    .child(
                        Button::new("icon-button-2", cx)
                            .icon(IconName::Close)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-3", cx)
                            .icon(IconName::Search)
                            .size(Size::Small)
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
                            .size(Size::Small),
                    )
                    .child(
                        Button::new("icon-button-5", cx)
                            .icon(IconName::Close)
                            .size(Size::Small)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-6", cx)
                            .icon(IconName::Search)
                            .size(Size::XSmall)
                            .style(ButtonStyle::Primary),
                    )
                    .child(
                        Button::new("icon-button-7", cx)
                            .icon(IconName::Info)
                            .size(Size::XSmall),
                    )
                    .child(
                        Button::new("icon-button-8", cx)
                            .icon(IconName::Close)
                            .size(Size::XSmall)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-9", cx)
                            .icon(IconName::Heart)
                            .size(px(24.))
                            .style(ButtonStyle::Ghost),
                    ),
            )
    }
}
