use gpui::{px, ParentElement as _, Render, Styled, View, VisualContext as _, WindowContext};
use ui::{h_flex, v_flex, Chart, SvgImg};

const GOOGLE_LOGO: &str = include_str!("./fixtures/google.svg");
const PIE_JSON: &str = include_str!("./fixtures/pie.json");

pub struct ImageStory {
    google_logo: SvgImg,
    pie_chart: Chart,
}

impl ImageStory {
    pub fn new(cx: &WindowContext) -> Self {
        Self {
            google_logo: SvgImg::new(800, 800)
                .svg(GOOGLE_LOGO.as_bytes(), cx)
                .unwrap(),
            pie_chart: Chart::new(ui::ChartKind::Pie, 800, 600, PIE_JSON, cx).unwrap(),
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
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.)).flex_grow())
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.)).flex_grow())
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.)).flex_grow())
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.)).flex_grow())
                    .child(self.google_logo.clone().w(px(300.)).h(px(300.)).flex_grow()),
            )
            .child(self.pie_chart.clone().size_full())
    }
}
