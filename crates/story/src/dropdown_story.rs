use gpui::{
    px, IntoElement, ParentElement, Render, SharedString, Styled, View, ViewContext, VisualContext,
    WindowContext,
};

use ui::{
    dropdown::{Dropdown, DropdownEvent, DropdownItem, SearchableVec},
    h_flex,
    theme::ActiveTheme,
    v_flex, Selection,
};

struct Country {
    name: SharedString,
    code: SharedString,
}

impl Country {
    pub fn new(name: impl Into<SharedString>, code: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
        }
    }
}

impl DropdownItem for Country {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.code
    }
}

pub struct DropdownStory {
    country_dropdown: View<Dropdown<Vec<Country>>>,
    fruit_dropdown: View<Dropdown<SearchableVec<SharedString>>>,
    simple_dropdown1: View<Dropdown<Vec<SharedString>>>,
    simple_dropdown2: View<Dropdown<Vec<SharedString>>>,
    simple_dropdown3: View<Dropdown<Vec<SharedString>>>,
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
            Dropdown::new("dropdown-country", countries, Some(6), cx).cleanable(true)
        });

        let fruits = SearchableVec::new(vec![
            "Apple".into(),
            "Orange".into(),
            "Banana".into(),
            "Grape".into(),
            "Pineapple".into(),
            "Watermelon & This is a longlonglonglonglonglonglonglonglong title".into(),
            "Avocado".into(),
        ]);
        let fruit_dropdown = cx.new_view(|cx| Dropdown::new("dropdown-fruits", fruits, None, cx));

        cx.new_view(|cx| {
            cx.subscribe(&country_dropdown, Self::on_dropdown_event)
                .detach();

            Self {
                country_dropdown,
                fruit_dropdown,
                simple_dropdown1: cx.new_view(|cx| {
                    Dropdown::new(
                        "string-list1",
                        vec!["QPUI".into(), "Iced".into(), "QT".into(), "Cocoa".into()],
                        Some(0),
                        cx,
                    )
                    .size(ui::Size::Small)
                    .placeholder("UI")
                    .title_prefix("UI: ")
                }),
                simple_dropdown2: cx.new_view(|cx| {
                    Dropdown::new(
                        "string-list2",
                        vec![
                            "Rust".into(),
                            "Go".into(),
                            "C++".into(),
                            "JavaScript".into(),
                        ],
                        None,
                        cx,
                    )
                    .size(ui::Size::Small)
                    .placeholder("Language")
                    .title_prefix("Language: ")
                }),
                simple_dropdown3: cx.new_view(|cx| {
                    Dropdown::new("string-list3", Vec::<SharedString>::new(), None, cx)
                        .size(ui::Size::Small)
                        .empty(|cx| {
                            h_flex()
                                .h_24()
                                .justify_center()
                                .text_color(cx.theme().muted_foreground)
                                .child("No Data")
                        })
                }),
            }
        })
    }

    #[allow(unused)]
    fn on_click(sel: &Selection, cx: &mut WindowContext) {
        println!("Check value changed: {}", sel);
    }

    fn on_dropdown_event(
        &mut self,
        _: View<Dropdown<Vec<Country>>>,
        event: &DropdownEvent<Vec<Country>>,
        _cx: &mut ViewContext<Self>,
    ) {
        match event {
            DropdownEvent::Confirm(value) => println!("Selected country: {:?}", value),
        }
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
                    .child(self.fruit_dropdown.clone()),
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
                        "fruit: {:?}",
                        self.fruit_dropdown.read(cx).selected_value()
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
                    .child(self.simple_dropdown2.clone())
                    .child(self.simple_dropdown3.clone()),
            )
    }
}
