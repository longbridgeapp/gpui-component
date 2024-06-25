use gpui::{
    div, ClickEvent, IntoElement, ParentElement, Render, RenderOnce,
    StatefulInteractiveElement as _, Styled, ViewContext, VisualContext as _, WindowContext,
};

use crate::{
    checkbox::Checkbox,
    disableable::Disableable as _,
    selectable::Selection,
    stock::{h_flex, v_flex},
    switch::Switch,
};

use super::story_case;

#[derive(Default)]
pub struct SwitchStory {
    switch1: bool,
    switch2: bool,
}

impl SwitchStory {
    pub(crate) fn new(cx: &mut WindowContext) -> Self {
        Self {
            switch1: false,
            switch2: true,
        }
    }
}

impl Render for SwitchStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        story_case(
            "Switch",
            "A control that allows the user to toggle between checked and not checked.",
        )
        .child(
            v_flex()
                .items_start()
                .justify_center()
                .gap_4()
                .child(
                    h_flex()
                        .items_center()
                        .gap_4()
                        .child(
                            Switch::new("switch1")
                                .checked(self.switch1)
                                .on_click(cx.listener(move |view, _, cx| {
                                    view.switch1 = !view.switch1;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Switch::new("switch2")
                                .checked(self.switch2)
                                .on_click(cx.listener(move |view, _, cx| {
                                    view.switch2 = !view.switch2;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_4()
                        .child(Switch::new("switch3").disabled(true).on_click(|ev, cx| {
                            println!("Switch value changed: {:?}", ev);
                        }))
                        .child(
                            Switch::new("switch3_1")
                                .checked(true)
                                .disabled(true)
                                .on_click(|ev, cx| {
                                    println!("Switch value changed: {:?}", ev);
                                }),
                        ),
                ),
        )
    }
}
