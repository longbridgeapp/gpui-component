use gpui::{px, rems, ParentElement, Render, Styled, View, VisualContext as _, WindowContext};
use ui::{
    button::{Button, ButtonStyle},
    h_flex,
    theme::ActiveTheme as _,
    v_flex, Icon, IconName,
};

pub struct IconStory {}

impl IconStory {
    pub fn new(_: &WindowContext) -> Self {
        Self {}
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }
}

impl Render for IconStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        v_flex().gap_3().child(
            h_flex()
                .gap_4()
                .child(IconName::Info)
                .child(
                    Icon::new(IconName::Maximize)
                        .size_6()
                        .text_color(ui::green_500()),
                )
                .child(Icon::new(IconName::Maximize).size(px(55.)))
                .child(
                    Button::new("like1", cx)
                        .icon(
                            Icon::new(IconName::Heart)
                                .text_color(ui::gray_500())
                                .size_6(),
                        )
                        .style(ButtonStyle::Ghost),
                )
                .child(
                    Button::new("like2", cx)
                        .icon(
                            Icon::new(IconName::HeartOff)
                                .text_color(ui::red_500())
                                .size_6(),
                        )
                        .style(ButtonStyle::Ghost),
                )
                .child(
                    Icon::new(IconName::Plus)
                        .w(rems(3.))
                        .h(rems(3.))
                        .bg(cx.theme().primary)
                        .text_color(cx.theme().primary_foreground)
                        .rounded(px(32.)),
                ),
        )
    }
}
