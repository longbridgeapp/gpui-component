use gpui::{
    anchored, deferred, div, prelude::FluentBuilder as _, px, AppContext, ElementId, EventEmitter,
    FocusHandle, FocusableView, Hsla, InteractiveElement as _, IntoElement, KeyBinding, Length,
    MouseButton, ParentElement, Render, SharedString, StatefulInteractiveElement as _, Styled,
    ViewContext,
};

use crate::{
    colors::DEFAULT_COLOR,
    divider::Divider,
    h_flex,
    input::ClearButton,
    popover::Escape,
    theme::{ActiveTheme as _, Colorize},
    v_flex, ColorExt as _, Icon, IconName, Size, StyleSized as _, StyledExt as _,
};

pub fn init(cx: &mut AppContext) {
    let context = Some("ColorPicker");
    cx.bind_keys([KeyBinding::new("escape", Escape, context)])
}

#[derive(Clone)]
pub enum ColorPickerEvent {
    Change(Option<Hsla>),
}

fn color_palettes() -> Vec<Vec<Hsla>> {
    use itertools::Itertools as _;

    macro_rules! c {
        ($color:tt) => {
            DEFAULT_COLOR
                .$color
                .keys()
                .sorted()
                .map(|k| DEFAULT_COLOR.$color.get(k).map(|c| c.hsla).unwrap())
                .collect::<Vec<_>>()
        };
    }

    vec![
        c!(stone),
        c!(red),
        c!(orange),
        c!(yellow),
        c!(green),
        c!(cyan),
        c!(blue),
        c!(purple),
        c!(pink),
    ]
}

pub struct ColorPicker {
    id: ElementId,
    focus_handle: FocusHandle,
    featured_colors: Vec<Hsla>,
    value: Option<Hsla>,
    cleanable: bool,
    open: bool,
    size: Size,
    width: Length,
    hovered_color: Option<Hsla>,
}

impl ColorPicker {
    pub fn new(id: impl Into<ElementId>, cx: &mut ViewContext<Self>) -> Self {
        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            featured_colors: vec![
                crate::black(),
                crate::white(),
                crate::red_600(),
                crate::orange_600(),
                crate::yellow_600(),
                crate::green_600(),
                crate::blue_600(),
                crate::indigo_600(),
                crate::purple_600(),
            ],
            value: None,
            cleanable: false,
            open: false,
            size: Size::default(),
            width: Length::Auto,
            hovered_color: None,
        }
    }

    /// Set true to show the clear button when the input field is not empty.
    pub fn cleanable(mut self) -> Self {
        self.cleanable = true;
        self
    }

    /// Set width of the date picker input field, default is `Length::Auto`.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn featured_colors(mut self, colors: Vec<Hsla>) -> Self {
        self.featured_colors = colors;
        self
    }

    pub fn value(mut self, value: Hsla) -> Self {
        self.value = Some(value);
        self
    }

    fn escape(&mut self, _: &Escape, cx: &mut ViewContext<Self>) {
        self.open = false;
        cx.notify();
    }

    fn clean(&mut self, _: &gpui::ClickEvent, cx: &mut ViewContext<Self>) {
        self.update_value(None, cx)
    }

    fn toggle_picker(&mut self, _: &gpui::ClickEvent, cx: &mut ViewContext<Self>) {
        self.open = !self.open;
        cx.notify();
    }

    fn update_value(&mut self, value: Option<Hsla>, cx: &mut ViewContext<Self>) {
        self.value = value;
        cx.emit(ColorPickerEvent::Change(value));
        cx.notify();
    }

    fn render_item(
        &self,
        color: Hsla,
        clickable: bool,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(format!(
                "color-{}",
                color.to_hex_string()
            )))
            .h_5()
            .w_5()
            .bg(color)
            .rounded_sm()
            .border_1()
            .border_color(color.darken(0.1))
            .when(clickable, |this| {
                this.cursor_pointer()
                    .hover(|this| this.border_color(color.darken(0.3)))
                    .on_mouse_move(cx.listener(move |view, _, cx| {
                        view.hovered_color = Some(color);
                        cx.notify();
                    }))
                    .on_click(cx.listener(move |view, _, cx| {
                        view.update_value(Some(color), cx);
                        view.open = false;
                        cx.notify();
                    }))
            })
    }

    fn render_colors(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                h_flex().gap_1().children(
                    self.featured_colors
                        .iter()
                        .map(|color| self.render_item(*color, true, cx)),
                ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .children(color_palettes().iter().map(|sub_colors| {
                        h_flex().gap_1().children(
                            sub_colors
                                .iter()
                                .rev()
                                .map(|color| self.render_item(*color, true, cx)),
                        )
                    })),
            )
            .when_some(self.hovered_color.clone(), |this, hovered_color| {
                this.child(Divider::horizontal()).child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            div()
                                .bg(hovered_color)
                                .size_5()
                                .rounded(px(cx.theme().radius)),
                        )
                        .child(hovered_color.to_hex_string()),
                )
            })
    }
}

impl EventEmitter<ColorPickerEvent> for ColorPicker {}
impl FocusableView for ColorPicker {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ColorPicker {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(cx);
        let show_clean = self.cleanable && self.value.is_some();

        let display_title = if let Some(value) = self.value {
            format!("{}", value.to_hex_string())
        } else {
            "Select a color".to_string()
        };

        let value = self.value.unwrap_or_else(|| cx.theme().foreground);

        div()
            .id(self.id.clone())
            .key_context("ColorPicker")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::escape))
            .w_full()
            .relative()
            .map(|this| match self.width {
                Length::Definite(l) => this.flex_none().w(l),
                Length::Auto => this.w_full(),
            })
            .input_text_size(self.size)
            .child(
                div()
                    .id("color-picker-input")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().input)
                    .rounded(px(cx.theme().radius))
                    .shadow_sm()
                    .cursor_pointer()
                    .overflow_hidden()
                    .input_text_size(self.size)
                    .when(is_focused, |this| this.outline(cx))
                    .input_size(self.size)
                    .when(!self.open, |this| {
                        this.on_click(cx.listener(Self::toggle_picker))
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_1()
                            .child(self.render_item(value, false, cx))
                            .child(div().flex_1().overflow_hidden().child(display_title))
                            .when(show_clean, |this| {
                                this.child(ClearButton::new(cx).on_click(cx.listener(Self::clean)))
                            })
                            .when(!show_clean, |this| {
                                this.child(
                                    Icon::new(IconName::Palette)
                                        .text_color(cx.theme().muted_foreground),
                                )
                            }),
                    ),
            )
            .when(self.open, |this| {
                this.child(
                    deferred(
                        anchored().snap_to_window().child(
                            div()
                                .track_focus(&self.focus_handle)
                                .occlude()
                                .absolute()
                                .mt_1p5()
                                .w_72()
                                .overflow_hidden()
                                .rounded_lg()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .shadow_lg()
                                .rounded_lg()
                                .bg(cx.theme().background)
                                .on_mouse_up_out(
                                    MouseButton::Left,
                                    cx.listener(|view, _, cx| view.escape(&Escape, cx)),
                                )
                                .child(self.render_colors(cx)),
                        ),
                    )
                    .with_priority(2),
                )
            })
    }
}
