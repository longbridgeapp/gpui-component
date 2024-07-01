use gpui::{
    div, px, ClickEvent, Div, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    View, ViewContext, VisualContext as _, WindowContext,
};

use ui::{
    button::{Button, ButtonSize, ButtonStyle},
    h_flex, v_flex, Clickable, Disableable as _, Icon, IconName, Selectable,
};

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
        fn section(title: impl Into<SharedString>, cx: &ViewContext<ButtonStory>) -> Div {
            use ui::theme::ActiveTheme;
            let theme = cx.theme();

            h_flex()
                .items_center()
                .gap_4()
                .p_4()
                .w_full()
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
                .flex_wrap()
                .justify_around()
                .child(div().flex_none().w_full().child(title.into()))
        }

        v_flex()
            .w_full()
            .justify_start()
            .gap_6()
            .child(
                section("Normal Button", cx)
                    .child(
                        Button::new("button-1")
                            .label("Primary Button")
                            .style(ButtonStyle::Primary)
                            .on_click(Self::on_click),
                    )
                    .child(
                        Button::new("button-2")
                            .label("Secondary Button")
                            .style(ButtonStyle::Secondary)
                            .on_click(Self::on_click),
                    )
                    .child(
                        Button::new("button-4")
                            .label("Danger Button")
                            .style(ButtonStyle::Danger)
                            .on_click(Self::on_click),
                    ),
            )
            .child(
                section("Button with Icon", cx)
                    .child(
                        Button::new("button-icon-1")
                            .label("Confirm")
                            .icon(IconName::Check)
                            .style(ButtonStyle::Primary)
                            .on_click(Self::on_click),
                    )
                    .child(
                        Button::new("button-icon-2")
                            .label("Abort")
                            .icon(IconName::Close)
                            .style(ButtonStyle::Secondary)
                            .on_click(Self::on_click),
                    )
                    .child(
                        Button::new("button-icon-3")
                            .label("Maximize")
                            .icon(Icon::new(IconName::Maximize))
                            .style(ButtonStyle::Secondary)
                            .on_click(Self::on_click),
                    ),
            )
            .child(
                h_flex()
                    .gap_6()
                    .child(
                        section("Small Size", cx)
                            .child(
                                Button::new("button-6")
                                    .label("Primary Button")
                                    .style(ButtonStyle::Primary)
                                    .size(ButtonSize::Small)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-7")
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .size(ButtonSize::Small)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-8")
                                    .label("Danger Button")
                                    .style(ButtonStyle::Danger)
                                    .size(ButtonSize::Small)
                                    .on_click(Self::on_click),
                            ),
                    )
                    .child(
                        section("XSmall Size", cx)
                            .child(
                                Button::new("button-xs-1")
                                    .label("Primary Button")
                                    .style(ButtonStyle::Primary)
                                    .size(ButtonSize::XSmall)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-2")
                                    .label("Secondary Button")
                                    .style(ButtonStyle::Secondary)
                                    .size(ButtonSize::XSmall)
                                    .on_click(Self::on_click),
                            )
                            .child(
                                Button::new("button-xs-3")
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
                                Button::new("button-disabled1")
                                    .label("Disabled Button")
                                    .style(ButtonStyle::Primary)
                                    .on_click(Self::on_click)
                                    .disabled(true),
                            )
                            .child(
                                Button::new("button-disabled1")
                                    .label("Disabled Button")
                                    .style(ButtonStyle::Secondary)
                                    .on_click(Self::on_click)
                                    .disabled(true),
                            )
                            .child(
                                Button::new("button-disabled1")
                                    .label("Disabled Button")
                                    .style(ButtonStyle::Danger)
                                    .on_click(Self::on_click)
                                    .disabled(true),
                            ),
                    )
                    .child(
                        section("Selected Style", cx)
                            .child(
                                Button::new("button-selected-1")
                                    .label("Selected Button")
                                    .style(ButtonStyle::Primary)
                                    .selected(true),
                            )
                            .child(
                                Button::new("button-selected-2")
                                    .label("Selected Button")
                                    .style(ButtonStyle::Secondary)
                                    .selected(true),
                            )
                            .child(
                                Button::new("button-selected-3")
                                    .label("Selected Button")
                                    .style(ButtonStyle::Danger)
                                    .selected(true),
                            ),
                    ),
            )
            .child(
                section("Icon Button", cx)
                    .child(
                        Button::new("icon-button-1")
                            .icon(IconName::Search)
                            .style(ButtonStyle::Primary),
                    )
                    .child(Button::new("button-7").icon(IconName::Info).selected(true))
                    .child(
                        Button::new("icon-button-2")
                            .icon(IconName::Close)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-3")
                            .icon(IconName::Search)
                            .size(ButtonSize::Small)
                            .style(ButtonStyle::Primary),
                    )
                    .child(
                        Button::new("icon-button-4")
                            .icon(IconName::Info)
                            .size(ButtonSize::Small)
                            .selected(true),
                    )
                    .child(
                        Button::new("icon-button-5")
                            .icon(IconName::Close)
                            .size(ButtonSize::Small)
                            .style(ButtonStyle::Danger),
                    )
                    .child(
                        Button::new("icon-button-6")
                            .icon(IconName::Search)
                            .size(ButtonSize::XSmall)
                            .style(ButtonStyle::Primary),
                    )
                    .child(
                        Button::new("icon-button-7")
                            .icon(IconName::Info)
                            .size(ButtonSize::XSmall)
                            .selected(true),
                    )
                    .child(
                        Button::new("icon-button-8")
                            .icon(IconName::Close)
                            .size(ButtonSize::XSmall)
                            .style(ButtonStyle::Danger),
                    ),
            )
    }
}
