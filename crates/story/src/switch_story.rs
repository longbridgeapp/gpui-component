use gpui::{
    Div, IntoElement, ParentElement, Render, SharedString, Styled, View, ViewContext,
    VisualContext as _, WindowContext,
};

use ui::{
    h_flex,
    label::Label,
    switch::{LabelSide, Switch},
    theme::ActiveTheme,
    v_flex, Disableable as _, Sizable, StyledExt,
};

pub struct SwitchStory {
    focus_handle: gpui::FocusHandle,
    switch1: bool,
    switch2: bool,
    switch3: bool,
}

impl super::Story for SwitchStory {
    fn title() -> &'static str {
        "Switch"
    }

    fn description() -> &'static str {
        "A control that allows the user to toggle between two states."
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }
}

impl SwitchStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            switch1: true,
            switch2: false,
            switch3: true,
        }
    }
}

impl gpui::FocusableView for SwitchStory {
    fn focus_handle(&self, _: &gpui::AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
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

        v_flex().gap_6()
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
                                .on_click(cx.listener(move |view, checked, cx| {
                                    view.switch1 = *checked;
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
                                .on_click(cx.listener(move |view, checked, cx| {
                                    view.switch2 = *checked;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    card(cx).v_flex()
                        .items_start().child(title("Disabled Switchs")).child(
                        h_flex().items_center()
                        .gap_6()
                        .child(Switch::new("switch3").disabled(true).on_click(|v, _| {
                            println!("Switch value changed: {:?}", v);
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
                    card(cx).v_flex()
                        .items_start().child(title("Small Switchs")).child(
                        h_flex().items_center()
                        .gap_6()
                        .child(Switch::new("switch3").checked(self.switch3).label("Small Size").small().on_click(cx.listener(move |view, checked, cx| {
                            view.switch3 = *checked;
                            cx.notify();
                        })),
                    )
                ),
            )
        )
    }
}
