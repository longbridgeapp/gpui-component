use gpui::{
    Div, IntoElement, ParentElement, Render, SharedString, Styled, ViewContext, WindowContext,
};

use ui::{
    button::ButtonSize,
    h_flex,
    label::Label,
    switch::{LabelSide, Switch},
    theme::ActiveTheme,
    v_flex, Disableable as _, StyledExt,
};

use super::story_case;

#[derive(Default)]
pub struct SwitchStory {
    switch1: bool,
    switch2: bool,
    switch3: bool,
}

impl SwitchStory {
    pub(crate) fn new(_cx: &mut WindowContext) -> Self {
        Self {
            switch1: false,
            switch2: true,
            switch3: true,
        }
    }
}

impl Render for SwitchStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.theme();

        fn title(title: impl Into<SharedString>) -> Div {
            v_flex().flex_1().gap_2().child(Label::new(title).text_xl())
        }

        fn card(cx: &ViewContext<SwitchStory>) -> Div {
            let theme = cx.theme();

            h_flex()
                .items_center()
                .gap_4()
                .p_4()
                .w_full()
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
        }

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
                    card(cx)
                    .child(
                        title("Marketing emails").child(
                            Label::new("Receive emails about new products, features, and more.").text_color(theme.muted_foreground)
                        )
                    )
                        .child(
                            Switch::new("switch1")
                                .checked(self.switch1)
                                .label_side(LabelSide::Left)
                                .label("Subscribe")
                                .on_click(cx.listener(move |view, _, cx| {
                                    view.switch1 = !view.switch1;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    card(cx)
                    .child(
                        title("Security emails").child(
                            Label::new("Receive emails about your account security. When turn off, you never recive email again.").text_color(theme.muted_foreground)
                        )
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
                    card(cx).v_flex()         .items_start().child(title("Disabled Switchs")).child(
                        h_flex().items_center()
                        .gap_6()
                        .child(Switch::new("switch3").disabled(true).on_click(|ev, _| {
                            println!("Switch value changed: {:?}", ev);
                        }))
                        .child(
                            Switch::new("switch3_1").label("Airplane Mode")
                                .checked(true)
                                .disabled(true)
                                .on_click(|ev, _| {
                                    println!("Switch value changed: {:?}", ev);
                                }),
                        ))
                )
                .child(
                    card(cx).v_flex()         .items_start().child(title("Disabled Switchs")).child(
                        h_flex().items_center()
                        .gap_6()
                        .child(Switch::new("switch3").checked(self.switch3).label("Small Size").size(ButtonSize::Small).on_click(cx.listener(move |view, _, cx| {
                            view.switch3 = !view.switch3;
                            cx.notify();
                        })),
                    )
                ),
            )
        )
    }
}
