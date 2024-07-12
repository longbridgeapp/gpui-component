use gpui::{
    IntoElement, ParentElement, Render, Styled, View, ViewContext, VisualContext as _,
    WindowContext,
};

use ui::{checkbox::Checkbox, h_flex, radio::Radio, v_flex, Disableable as _, Selection};

pub struct CheckboxStory {
    check1: Selection,
    check2: Selection,
    check3: Selection,
    select1: bool,
    select2: bool,
}

impl CheckboxStory {
    pub(crate) fn new(_cx: &mut WindowContext) -> Self {
        Self {
            check1: Selection::Unselected,
            check2: Selection::Indeterminate,
            check3: Selection::Selected,
            select1: false,
            select2: true,
        }
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self::new(cx))
    }

    #[allow(unused)]
    fn on_click(sel: &Selection, cx: &mut WindowContext) {
        println!("Check value changed: {}", sel);
    }
}

impl Render for CheckboxStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .p_4()
            .gap_6()
            .child(
                v_flex().items_start().justify_start().gap_6().child(
                    h_flex()
                        .items_center()
                        .gap_6()
                        .child(
                            Checkbox::new("check1")
                                .checked(self.check1)
                                .on_click(cx.listener(|v, _, _| {
                                    v.check1 = v.check1.inverse();
                                })),
                        )
                        .child(
                            Checkbox::new("check2")
                                .checked(self.check2)
                                .label("Subscribe to newsletter")
                                .on_click(cx.listener(|v, _, _| {
                                    v.check2 = v.check2.inverse();
                                })),
                        )
                        .child(
                            Checkbox::new("check3")
                                .checked(self.check3)
                                .label("Remember me")
                                .on_click(cx.listener(|v, _, _| {
                                    v.check3 = v.check3.inverse();
                                })),
                        ),
                ),
            )
            .child(
                h_flex().items_center().gap_4().child(
                    h_flex()
                        .items_center()
                        .gap_6()
                        .child(
                            Checkbox::new("check3")
                                .label("Disabled Checked")
                                .checked(Selection::Selected)
                                .disabled(true),
                        )
                        .child(
                            Checkbox::new("check3_1")
                                .label("Disabled Unchecked")
                                .checked(Selection::Unselected)
                                .disabled(true),
                        )
                        .child(
                            Checkbox::new("check3_2")
                                .label("Disabled Indeterminate")
                                .checked(Selection::Indeterminate)
                                .disabled(true),
                        ),
                ),
            )
            .child(
                h_flex()
                    .gap_4()
                    .child(
                        Radio::new("radio1")
                            .selected(self.select1)
                            .on_click(cx.listener(|this, v, _cx| {
                                this.select1 = *v;
                            })),
                    )
                    .child(
                        Radio::new("radio2")
                            .label("Radio")
                            .selected(self.select2)
                            .on_click(cx.listener(|this, v, _cx| {
                                this.select2 = *v;
                            })),
                    )
                    .child(
                        Radio::new("radio3")
                            .label("Disabled Radio")
                            .selected(true)
                            .disabled(true),
                    ),
            )
    }
}
