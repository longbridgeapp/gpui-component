use gpui::{
    div, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};
use ui::{calendar::Calendar, h_flex, theme::ActiveTheme as _};

pub struct CalendarStory {
    calendar1: View<Calendar>,
}

impl CalendarStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let calendar1 = cx.new_view(Calendar::new);

        Self { calendar1 }
    }
}

impl Render for CalendarStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex().child(
            div()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .p_3()
                .child(self.calendar1.clone()),
        )
    }
}
