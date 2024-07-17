use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext,
    WindowContext,
};
use ui::{
    button::Button, divider::Divider, h_flex, indicator::Indicator, progress::Progress,
    slider::Slider, v_flex, Clickable, IconName, Size,
};

pub struct ProgressStory {
    value: f32,
    slider1: View<Slider>,
    slider1_value: f32,
    slider2: View<Slider>,
    slider2_value: f32,
}

impl ProgressStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let slider1 = Slider::horizontal()
            .min(-255.)
            .max(255.)
            .default_value(15.)
            .step(15.)
            .on_change(cx.listener(|this, value, cx| {
                this.slider1_value = *value;
                cx.notify();
            }));

        let slider2 = Slider::horizontal()
            .min(0.)
            .max(5.)
            .step(1.0)
            .on_change(cx.listener(|this, value, cx| {
                this.slider2_value = *value;
                cx.notify();
            }));

        Self {
            value: 50.,
            slider1_value: 15.,
            slider2_value: 1.,
            slider1: cx.new_view(|_| slider1),
            slider2: cx.new_view(|_| slider2),
        }
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
            .child(Divider::horizontal().mt_10().label("Slider"))
            .child(self.slider1.clone())
            .child(format!("Slider 1: {}", self.slider1_value))
            .child(
                v_flex()
                    .gap_3()
                    .w(px(200.))
                    .child(self.slider2.clone())
                    .child(format!("Slider 2: {}", self.slider2_value)),
            )
    }
}
