use std::rc::Rc;

use gpui::{
    px, IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext, WindowContext,
};

use ui::{
    dropdown::{Dropdown, DropdownDelegate, DropdownItem, StringDropdownDelegate},
    h_flex,
    theme::ActiveTheme,
    v_flex, Selection,
};

struct Country {
    name: &'static str,
    code: &'static str,
}

impl Country {
    pub fn new(name: &'static str, code: &'static str) -> Self {
        Self { name, code }
    }
}

impl DropdownItem for &Country {
    fn title(&self) -> &str {
        self.name
    }

    fn value(&self) -> &str {
        self.code
    }
}

struct CounterDelegate(Vec<Country>);
struct FuritDelegate(Vec<String>);

pub struct DropdownStory {
    country_dropdown: View<Dropdown<CounterDelegate>>,
    furit_dropdown: View<Dropdown<FuritDelegate>>,
    simple_dropdown1: View<Dropdown<StringDropdownDelegate>>,
    simple_dropdown2: View<Dropdown<StringDropdownDelegate>>,
}

impl DropdownDelegate for CounterDelegate {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, ix: usize) -> Option<impl DropdownItem> {
        self.0.get(ix)
    }
}

impl DropdownDelegate for FuritDelegate {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, ix: usize) -> Option<impl DropdownItem> {
        self.0.get(ix)
    }
}

impl DropdownStory {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        let countries = CounterDelegate(vec![
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
        ]);

        let country_dropdown =
            cx.new_view(|cx| Dropdown::new("dropdown-country", countries, Some(6), cx));

        let furits = FuritDelegate(
            [
                "Apple",
                "Orange",
                "Banana",
                "Grape",
                "Pineapple",
                "Watermelon",
                "Avocado",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
        let furit_dropdown = cx.new_view(|cx| Dropdown::new("dropdown-furits", furits, None, cx));

        cx.new_view(|cx| Self {
            country_dropdown,
            furit_dropdown,
            simple_dropdown1: cx.new_view(|cx| {
                Dropdown::string_list(
                    "string-list1",
                    Rc::new(vec![
                        "QPUI".into(),
                        "Iced".into(),
                        "QT".into(),
                        "Cocoa".into(),
                    ]),
                    Some(0),
                    cx,
                )
                .size(ui::Size::Small)
            }),
            simple_dropdown2: cx.new_view(|cx| {
                Dropdown::string_list(
                    "string-list2",
                    Rc::new(vec![
                        "Rust".into(),
                        "Go".into(),
                        "C++".into(),
                        "JavaScript".into(),
                    ]),
                    None,
                    cx,
                )
                .size(ui::Size::Small)
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
                h_flex()
                    .w_full()
                    .items_center()
                    .p_10()
                    .rounded_lg()
                    .bg(cx.theme().card)
                    .border_1()
                    .border_color(cx.theme().border)
                    .gap_4()
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
