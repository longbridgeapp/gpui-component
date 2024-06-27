use std::{rc::Rc, vec};

use gpui::{
    div, ClickEvent, IntoElement, ParentElement, Render, RenderOnce, SharedString, Styled,
    ViewContext, WindowContext,
};

use ui::{
    checkbox::Checkbox,
    dropdown::{Dropdown, DropdownItem},
    h_flex, v_flex, Disableable as _, Selection,
};

use super::story_case;

struct Country {
    name: &'static str,
    code: &'static str,
}

impl Country {
    pub fn new(name: &'static str, code: &'static str) -> Self {
        Self { name, code }
    }
}

impl DropdownItem for Country {
    fn title(&self) -> SharedString {
        self.name.into()
    }

    fn value(&self) -> SharedString {
        self.code.into()
    }
}

#[derive(IntoElement)]
pub struct DropdownStory {
    dropdown1: Dropdown,
    dropdown2: Dropdown,
    countries: Vec<Country>,
}

impl DropdownStory {
    pub(crate) fn new(cx: &mut WindowContext) -> Self {
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

        let items2 = vec![
            "Apple",
            "Orange",
            "Banana",
            "Grape",
            "Pineapple",
            "Watermelon",
            "Avocado",
        ];

        Self {
            countries,
            dropdown1: Dropdown::new("dropdown-country", Rc::new(countries), cx),
            dropdown2: Dropdown::new("dropdown-fruit", Rc::new(items2), cx),
        }
    }

    #[allow(unused)]
    fn on_click(sel: &Selection, cx: &mut WindowContext) {
        println!("Check value changed: {}", sel);
    }
}

impl RenderOnce for CheckboxStory {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        story_case(
            "Checkbox",
            "A control that allows the user to toggle between checked and not checked.",
        )
        .child(
            v_flex().items_start().justify_start().gap_4().child(
                h_flex()
                    .items_center()
                    .gap_4()
                    .child(self.check1)
                    .child(self.check1_1)
                    .child(
                        Checkbox::new("check1_2", cx)
                            .checked(Selection::Selected)
                            .on_click(Self::on_click),
                    ),
            ),
        )
        .child(
            h_flex()
                .items_center()
                .gap_4()
                .child(
                    Checkbox::new("check2", cx)
                        .checked(Selection::Unselected)
                        .label("With label (Unchecked)")
                        .on_click(Self::on_click),
                )
                .child(
                    Checkbox::new("check2_1", cx)
                        .label("With Label (Indeterminate)")
                        .checked(Selection::Indeterminate)
                        .on_click(Self::on_click),
                )
                .child(
                    Checkbox::new("check2_2", cx)
                        .label("With Label (Checked)")
                        .checked(Selection::Selected)
                        .on_click(Self::on_click),
                ),
        )
        .child(
            h_flex().items_center().gap_4().child(
                h_flex()
                    .items_center()
                    .gap_4()
                    .child(
                        Checkbox::new("check3", cx)
                            .label("Disabled Checked")
                            .checked(Selection::Selected)
                            .disabled(true),
                    )
                    .child(
                        Checkbox::new("check3_1", cx)
                            .label("Disabled Unchecked")
                            .checked(Selection::Unselected)
                            .disabled(true),
                    )
                    .child(
                        Checkbox::new("check3_2", cx)
                            .label("Disabled Indeterminate")
                            .checked(Selection::Indeterminate)
                            .disabled(true),
                    ),
            ),
        )
    }
}
