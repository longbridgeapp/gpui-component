use gpui::{
    div, px, CursorStyle, InteractiveElement, ParentElement, Render, StatefulInteractiveElement,
    Styled,
};

use ui::{
    button::{Button, ButtonStyle},
    checkbox::Checkbox,
    h_flex,
    label::Label,
    tooltip::Tooltip,
    v_flex, Selection,
};

use crate::story_case;

pub struct TooltipStory;

impl Render for TooltipStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let description = "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.";

        story_case("Tooltip", description).child(
            v_flex()
                .w(px(360.))
                .gap_5()
                .child(
                    div()
                        .cursor(CursorStyle::PointingHand)
                        .child(Button::new("button", "Hover me").style(ButtonStyle::Primary))
                        .id("tooltip-1")
                        .tooltip(|cx| Tooltip::text("This is a Button", cx)),
                )
                .child(
                    div()
                        .cursor(CursorStyle::PointingHand)
                        .child(
                            Button::new("button-meta", "With meta, Hover me")
                                .style(ButtonStyle::Primary),
                        )
                        .id("tooltip-2")
                        .tooltip(|cx| {
                            Tooltip::with_meta("This is a Button", "Click if you want", cx)
                        }),
                )
                .child(
                    h_flex()
                        .justify_center()
                        .cursor(CursorStyle::PointingHand)
                        .child(Label::new("Hover me"))
                        .id("tooltip-3")
                        .tooltip(|cx| Tooltip::text("This is a Label", cx)),
                )
                .child(
                    div()
                        .cursor(CursorStyle::PointingHand)
                        .child(
                            Checkbox::new("check", cx)
                                .label("Remember me")
                                .checked(Selection::Selected),
                        )
                        .id("tooltip-4")
                        .tooltip(|cx| Tooltip::text("Checked!", cx)),
                ),
        )
    }
}
