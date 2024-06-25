use gpui::{
    div, ClickEvent, IntoElement, ParentElement, Render, RenderOnce, Styled, ViewContext,
    WindowContext,
};

use crate::{
    checkbox::Checkbox,
    disableable::Disableable as _,
    selectable::Selection,
    stock::{h_flex, v_flex},
};

use super::story_case;

#[derive(IntoElement)]
pub struct CheckboxStory {
    check1: Checkbox,
    check1_1: Checkbox,
}

impl CheckboxStory {
    pub(crate) fn new(cx: &mut WindowContext) -> Self {
        Self {
            check1: Checkbox::new("check1", cx)
                .checked(Selection::Unselected)
                .on_click(Self::on_click),
            check1_1: Checkbox::new("check1_1", cx)
                .checked(Selection::Indeterminate)
                .on_click(Self::on_click),
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
