use std::borrow::Cow;

use gpui::{
    px, IntoElement, ParentElement, Render, SharedString, Styled, View, ViewContext, VisualContext,
    WindowContext,
};

use ui::{
    dropdown::{Dropdown, DropdownItem},
    h_flex,
    theme::ActiveTheme,
    v_flex, Selection,
};

struct Country {
    name: String,
    code: String,
}

impl Country {
    pub fn new(name: &str, code: &str) -> Self {
        Self {
            name: name.to_string(),
            code: code.to_string(),
        }
    }
}

impl DropdownItem for Country {
    type Value = String;

    fn title(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }

    fn value(&self) -> &Self::Value {
        &self.code
    }
}

pub struct DropdownStory {
    country_dropdown: View<Dropdown<Vec<Country>>>,
    furit_dropdown: View<Dropdown<Vec<SharedString>>>,
    simple_dropdown1: View<Dropdown<Vec<SharedString>>>,
    simple_dropdown2: View<Dropdown<Vec<SharedString>>>,
}

impl DropdownStory {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        let countries = vec![
            Country::new("United States", "US"),
            Country::new("Canada", "CA"),
            Country::new("Mexico", "MX"),
            Country::new("Brazil", "BR"),
            Country::new("Argentina", "AR"),
            Country::new("Chile", "CL"),
            Country::new("China", "CN"),
            Country::new("Peru", "PE"),
            Country::new("Colombia", "CO"),
            Country::new("Venezuela", "VE"),
            Country::new("Ecuador", "EC"),
        ];

        let country_dropdown = cx.new_view(|cx| {
            Dropdown::new("dropdown-country", countries, Some(6), cx).on_change(|value, _cx| {
                println!("Country changed: {:?}", value);
            })
        });

        let furits = vec![
            "Apple",
            "Orange",
            "Banana",
            "Grape",
            "Pineapple",
            "Watermelon",
            "Avocado",
        ];
        let furit_dropdown = cx.new_view(|cx| {
            Dropdown::string_list("dropdown-furits", furits, None, cx).on_change(|value, _cx| {
                println!("Furit changed: {:?}", value);
            })
        });

        cx.new_view(|cx| Self {
            country_dropdown,
            furit_dropdown,
            simple_dropdown1: cx.new_view(|cx| {
                Dropdown::string_list(
                    "string-list1",
                    vec!["QPUI", "Iced", "QT", "Cocoa"],
                    Some(0),
                    cx,
                )
                .size(ui::Size::Small)
                .placeholder("UI")
                .title_prefix("UI: ")
            }),
            simple_dropdown2: cx.new_view(|cx| {
                Dropdown::string_list(
                    "string-list2",
                    vec!["Rust", "Go", "C++", "JavaScript"],
                    None,
                    cx,
                )
                .size(ui::Size::Small)
                .placeholder("Language")
                .title_prefix("Language: ")
            }),
        })
    }

    #[allow(unused)]
    fn on_click(sel: &Selection, cx: &mut WindowContext) {
        println!("Check value changed: {}", sel);
    }
}

impl Render for DropdownStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .child(
                h_flex()
                    .w_full()
                    .max_w(px(640.))
                    .items_center()
                    .gap_4()
                    .child(self.country_dropdown.clone())
                    .child(self.furit_dropdown.clone()),
            )
            .child(
                v_flex()
                    .w_full()
                    .items_center()
                    .p_10()
                    .rounded_lg()
                    .bg(cx.theme().card)
                    .border_1()
                    .border_color(cx.theme().border)
                    .gap_4()
                    .child(format!(
                        "Country: {:?}",
                        self.country_dropdown.read(cx).selected_value()
                    ))
                    .child(format!(
                        "Furit: {:?}",
                        self.furit_dropdown.read(cx).selected_value()
                    ))
                    .child(format!(
                        "UI: {:?}",
                        self.simple_dropdown1.read(cx).selected_value()
                    ))
                    .child(format!(
                        "Language: {:?}",
                        self.simple_dropdown2.read(cx).selected_value()
                    ))
                    .child("This is other text."),
            )
            .child(
                h_flex()
                    .items_center()
                    .w_128()
                    .gap_2()
                    .child(self.simple_dropdown1.clone())
                    .child(self.simple_dropdown2.clone()),
            )
    }
}
