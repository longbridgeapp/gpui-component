use chrono::Days;
use gpui::{
    px, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};
use ui::{
    date_picker::{DatePicker, DatePickerEvent},
    v_flex, Sizable as _,
};

pub struct CalendarStory {
    date_picker: View<DatePicker>,
    date_picker_small: View<DatePicker>,
    date_picker_large: View<DatePicker>,
    date_picker_value: Option<String>,
    date_range_picker: View<DatePicker>,
    default_range_mode_picker: View<DatePicker>,
}

impl super::Story for CalendarStory {
    fn title() -> &'static str {
        "Calendar"
    }

    fn description() -> &'static str {
        "A date picker and calendar component."
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }
}

impl CalendarStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        let now = chrono::Local::now().naive_local().date();
        let date_picker = cx.new_view(|cx| {
            let mut picker = DatePicker::new("date_picker_medium", cx)
                .cleanable()
                .width(px(220.));
            picker.set_date(now, cx);
            picker
        });
        let date_picker_large = cx.new_view(|cx| {
            DatePicker::new("date_picker_large", cx)
                .large()
                .date_format("%Y-%m-%d")
                .width(px(300.))
        });
        let date_picker_small = cx.new_view(|cx| {
            let mut picker = DatePicker::new("date_picker_small", cx)
                .small()
                .width(px(180.));
            picker.set_date(now, cx);
            picker
        });
        let date_range_picker = cx.new_view(|cx| {
            let mut picker = DatePicker::new("date_range_picker", cx)
                .width(px(300.))
                .number_of_months(2)
                .cleanable();
            picker.set_date((now, now.checked_add_days(Days::new(4)).unwrap()), cx);
            picker
        });

        cx.subscribe(&date_picker, |this, _, ev, _| match ev {
            DatePickerEvent::Change(date) => {
                this.date_picker_value = date.format("%Y-%m-%d").map(|s| s.to_string());
            }
        })
        .detach();
        cx.subscribe(&date_range_picker, |this, _, ev, _| match ev {
            DatePickerEvent::Change(date) => {
                this.date_picker_value = date.format("%Y-%m-%d").map(|s| s.to_string());
            }
        })
        .detach();

        let default_range_mode_picker = cx.new_view(|cx| {
            DatePicker::range_picker("default_range_mode_picker", cx)
                .width(px(300.))
                .placeholder("Range mode picker")
                .cleanable()
        });

        cx.subscribe(&default_range_mode_picker, |this, _, ev, _| match ev {
            DatePickerEvent::Change(date) => {
                this.date_picker_value = date.format("%Y-%m-%d").map(|s| s.to_string());
            }
        })
        .detach();

        Self {
            date_picker,
            date_picker_large,
            date_picker_small,
            date_range_picker,
            default_range_mode_picker,
            date_picker_value: None,
        }
    }
}

impl gpui::FocusableView for CalendarStory {
    fn focus_handle(&self, cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.date_picker.focus_handle(cx)
    }
}

impl Render for CalendarStory {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(self.date_picker.clone())
            .child(self.date_picker_small.clone())
            .child(self.date_picker_large.clone())
            .child(self.date_range_picker.clone())
            .child(self.default_range_mode_picker.clone())
            .child(format!("Date picker value: {:?}", self.date_picker_value).into_element())
    }
}
