use gpui::{
    FocusHandle, IntoElement, ParentElement as _, Render, Styled as _, View, ViewContext,
    VisualContext as _, WindowContext,
};
use ui::{
    accordion::Accordion, button::Button, button_group::ButtonGroup, checkbox::Checkbox,
    switch::Switch, v_flex, Selectable, Sizable, Size,
};

pub struct AccordionStory {
    expanded_ix: usize,
    size: Size,
    focus_handle: FocusHandle,
}

impl super::Story for AccordionStory {
    fn title() -> &'static str {
        "Accordion"
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }
}

impl AccordionStory {
    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            expanded_ix: 0,
            size: Size::default(),
            focus_handle: cx.focus_handle(),
        }
    }

    fn toggle_accordion(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        self.expanded_ix = ix;
        cx.notify();
    }

    fn set_size(&mut self, size: Size, cx: &mut ViewContext<Self>) {
        self.size = size;
        cx.notify();
    }
}

impl gpui::FocusableView for AccordionStory {
    fn focus_handle(&self, _: &gpui::AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AccordionStory {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(
                ButtonGroup::new("toggle-size")
                    .mb_3()
                    .small()
                    .child(
                        Button::new("xsmall")
                            .label("XSmall")
                            .selected(self.size == Size::XSmall),
                    )
                    .child(
                        Button::new("small")
                            .label("Small")
                            .selected(self.size == Size::Small),
                    )
                    .child(
                        Button::new("medium")
                            .label("Medium")
                            .selected(self.size == Size::Medium),
                    )
                    .child(
                        Button::new("large")
                            .label("Large")
                            .selected(self.size == Size::Large),
                    )
                    .on_click(cx.listener(|this, selected: &Vec<usize>, cx| {
                        this.set_size(Size::XSmall, cx);
                    })),
            )
            .child(
                Accordion::new()
                    .with_size(self.size)
                    .expanded(self.expanded_ix == 0)
                    .title("This is first accordion")
                    .content("Hello")
                    .on_toggle_click(cx.listener(|this, _, cx| {
                        this.toggle_accordion(0, cx);
                    })),
            )
            .child(
                Accordion::new()
                    .with_size(self.size)
                    .expanded(self.expanded_ix == 1)
                    .title("This is second accordion")
                    .content(
                        v_flex()
                            .gap_2()
                            .child("We can put any view here, like a v_flex with a text view")
                            .child(Switch::new("switch1").label("Switch"))
                            .child(Checkbox::new("checkbox1").label("Or a Checkbox")),
                    )
                    .on_toggle_click(cx.listener(|this, _, cx| {
                        this.toggle_accordion(1, cx);
                    })),
            )
    }
}
