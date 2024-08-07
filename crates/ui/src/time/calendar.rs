use std::borrow::Cow;

use chrono::{Datelike, Local, NaiveDate};
use gpui::{
    prelude::FluentBuilder as _, ClickEvent, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement, Styled as _, ViewContext,
};
use rust_i18n::t;

use crate::{
    button::Button,
    h_flex,
    theme::{ActiveTheme, Colorize},
    v_flex, Clickable, IconName, StyledExt,
};

use super::utils::days_in_month;

pub enum CalendarEvent {
    /// The user selected a date.
    Selected(NaiveDate),
}

pub struct Calendar {
    date: Option<NaiveDate>,
    current_year: i32,
    current_month: u8,
}

impl Calendar {
    pub fn new(_cx: &mut ViewContext<Self>) -> Self {
        let today = Local::now().naive_local().date();
        Self {
            date: None,
            current_month: today.month() as u8,
            current_year: today.year(),
        }
    }

    /// Set the date of the calendar.
    pub fn set_date(&mut self, date: Option<NaiveDate>, cx: &mut ViewContext<Self>) {
        self.date = date;
        if let Some(date) = date {
            self.current_month = date.month() as u8;
            self.current_year = date.year();
        }
        cx.notify()
    }

    /// Get the date of the calendar.
    pub fn date(&self) -> Option<NaiveDate> {
        self.date
    }

    /// Returns the days of the month in a 2D vector to render on calendar.
    fn days(&self) -> Vec<Vec<NaiveDate>> {
        days_in_month(self.current_year, self.current_month as u32)
    }

    fn prev_month(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.current_month = if self.current_month == 1 {
            12
        } else {
            self.current_month - 1
        };
        self.current_year = if self.current_month == 12 {
            self.current_year - 1
        } else {
            self.current_year
        };
        cx.notify()
    }

    fn next_month(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.current_month = if self.current_month == 12 {
            1
        } else {
            self.current_month + 1
        };
        self.current_year = if self.current_month == 1 {
            self.current_year + 1
        } else {
            self.current_year
        };
        cx.notify()
    }

    fn month_name(&self) -> SharedString {
        match self.current_month {
            1 => t!("Calendar.month.January"),
            2 => t!("Calendar.month.February"),
            3 => t!("Calendar.month.March"),
            4 => t!("Calendar.month.April"),
            5 => t!("Calendar.month.May"),
            6 => t!("Calendar.month.June"),
            7 => t!("Calendar.month.July"),
            8 => t!("Calendar.month.August"),
            9 => t!("Calendar.month.September"),
            10 => t!("Calendar.month.October"),
            11 => t!("Calendar.month.November"),
            12 => t!("Calendar.month.December"),
            _ => Cow::Borrowed(""),
        }
        .into()
    }

    fn render_week(
        &self,
        week: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        h_flex()
            .w_9()
            .h_9()
            .rounded_md()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .text_sm()
            .child(week.into())
    }

    fn render_item(
        &self,
        ix: usize,
        d: &NaiveDate,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        let day = d.day();
        let is_current_month = d.month() == self.current_month as u32;
        let is_active = match self.date {
            Some(date) => date == *d,
            None => false,
        };

        let date = *d;

        h_flex()
            .id(ix)
            .w_9()
            .h_9()
            .rounded_lg()
            .justify_center()
            .cursor_pointer()
            .when(!is_current_month, |this| {
                this.text_color(cx.theme().muted_foreground.opacity(0.3))
            })
            .when(!is_active, |this| {
                this.hover(|this| {
                    this.bg(cx.theme().accent)
                        .text_color(cx.theme().accent_foreground)
                })
            })
            .when(is_active, |this| {
                this.bg(cx.theme().primary)
                    .text_color(cx.theme().primary_foreground)
            })
            .on_click(cx.listener(move |view, _: &ClickEvent, cx| {
                view.set_date(Some(date), cx);
                cx.emit(CalendarEvent::Selected(date))
            }))
            .child(day.to_string())
    }
}

impl EventEmitter<CalendarEvent> for Calendar {}

impl Render for Calendar {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let weeks: [SharedString; 7] = [
            t!("Calendar.week.0").into(),
            t!("Calendar.week.1").into(),
            t!("Calendar.week.2").into(),
            t!("Calendar.week.3").into(),
            t!("Calendar.week.4").into(),
            t!("Calendar.week.5").into(),
            t!("Calendar.week.6").into(),
        ];

        v_flex().gap_0p5().text_sm().child(
            v_flex()
                .child(
                    h_flex()
                        .gap_0p5()
                        .justify_between()
                        .items_center()
                        .child(
                            Button::new("prev", cx)
                                .icon(IconName::ArrowLeft)
                                .ghost()
                                .on_click(cx.listener(Self::prev_month)),
                        )
                        .child(
                            h_flex()
                                .justify_center()
                                .font_semibold()
                                .gap_3()
                                // TODO: Add menu to select month and year
                                .child(self.current_year.to_string())
                                .child(self.month_name()),
                        )
                        .child(
                            Button::new("next", cx)
                                .icon(IconName::ArrowRight)
                                .ghost()
                                .on_click(cx.listener(Self::next_month)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_0p5()
                        .children(weeks.iter().map(|week| self.render_week(week.clone(), cx))),
                )
                .children(self.days().iter().map(|week| {
                    h_flex().gap_0p5().children(
                        week.iter()
                            .enumerate()
                            .map(|(ix, d)| self.render_item(ix, d, cx)),
                    )
                })),
        )
    }
}
