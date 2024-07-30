use gpui::{px, ParentElement as _, Render, Styled, View, VisualContext as _, WindowContext};
use ui::{h_flex, svg_img, v_flex, SvgImg};

const GOOGLE_LOGO: &str = include_str!("./fixtures/google.svg");
const PIE_JSON: &str = include_str!("./fixtures/pie.json");

pub struct ImageStory {
    google_logo: SvgImg,
    pie_chart: SvgImg,
    inbox_img: SvgImg,
}

impl ImageStory {
    pub fn new(_: &WindowContext) -> Self {
        let chart = charts_rs::PieChart::from_json(PIE_JSON).unwrap();

        Self {
            google_logo: svg_img().source(GOOGLE_LOGO.as_bytes(), px(300.), px(300.)),
            pie_chart: svg_img().source(chart.svg().unwrap().as_bytes(), px(400.), px(400.)),
            inbox_img: svg_img().source("icons/inbox.svg", px(300.), px(300.)),
        }
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }
}

impl Render for ImageStory {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        v_flex()
            .gap_4()
            .size_full()
            .items_center()
            .child(
                h_flex()
                    .size_full()
                    .child(self.google_logo.clone().size(px(300.)).flex_grow())
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.)).flex_grow())
                    .child(self.google_logo.clone().size_80().flex_grow())
                    .child(self.google_logo.clone().size_12().flex_grow())
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.))),
            )
            .child(self.inbox_img.clone().w(px(80.)).h(px(80.)))
            .child(self.pie_chart.clone().size_full())
    }
}
