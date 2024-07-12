use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext,
    WindowContext,
};
use ui::{
    button::Button, h_flex, indicator::Indicator, progress::Progress, v_flex, Clickable, IconName,
    Size,
};

pub struct ProgressStory {
    value: f32,
}

impl ProgressStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self { value: 50. })
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}

impl Render for ProgressStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .items_center()
            .gap_y_3()
            .child(
                h_flex()
                    .gap_x_2()
                    .child(
                        Button::new("button-1", cx)
                            .label("0%")
                            .on_click(cx.listener(|this, _, _| {
                                this.set_value(0.);
                            })),
                    )
                    .child(
                        Button::new("button-2", cx)
                            .label("25%")
                            .on_click(cx.listener(|this, _, _| {
                                this.set_value(25.);
                            })),
                    )
                    .child(
                        Button::new("button-3", cx)
                            .label("75%")
                            .on_click(cx.listener(|this, _, _| {
                                this.set_value(75.);
                            })),
                    )
                    .child(
                        Button::new("button-4", cx)
                            .label("100%")
                            .on_click(cx.listener(|this, _, _| {
                                this.set_value(100.);
                            })),
                    ),
            )
            .child(div().w_1_2().child(Progress::new().value(self.value)))
            .child(
                h_flex()
                    .gap_x_2()
                    .child(
                        Button::new("button-5", cx)
                            .icon(IconName::Minus)
                            .on_click(cx.listener(|this, _, _| {
                                this.set_value((this.value - 1.).max(0.));
                            })),
                    )
                    .child(
                        Button::new("button-6", cx)
                            .icon(IconName::Plus)
                            .on_click(cx.listener(|this, _, _| {
                                this.set_value((this.value + 1.).min(100.));
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_x_2()
                    .child(Indicator::new().size(Size::XSmall))
                    .child(Indicator::new().size(Size::Small))
                    .child(Indicator::new())
                    .child(
                        Indicator::new()
                            .size(Size::Large)
                            .icon(IconName::LoaderCircle)
                            .color(ui::blue_500()),
                    )
                    .child(Indicator::new().size(px(64.))),
            )
    }
}
