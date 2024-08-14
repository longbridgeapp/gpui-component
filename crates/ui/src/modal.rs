use std::{rc::Rc, time::Duration};

use gpui::{
    anchored, div, hsla, prelude::FluentBuilder, px, Animation, AnimationExt as _, AnyElement,
    Bounds, ClickEvent, Div, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Pixels, Point, RenderOnce, Styled, WindowContext,
};

use crate::{
    button::Button, theme::ActiveTheme as _, v_flex, ContextModal, IconName, Sizable as _,
};

#[derive(IntoElement)]
pub struct Modal {
    base: Div,
    title: Option<AnyElement>,
    footer: Option<AnyElement>,
    content: Div,
    width: Pixels,
    max_width: Option<Pixels>,
    margin_top: Option<Pixels>,
    on_close: Rc<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>,
    show_close: bool,
    overlay: bool,
}

pub(crate) fn overlay_color(overlay: bool, cx: &WindowContext) -> Hsla {
    if !overlay {
        return hsla(0., 0., 0., 0.);
    }

    if cx.theme().mode.is_dark() {
        hsla(0., 1., 1., 0.06)
    } else {
        hsla(0., 0., 0., 0.06)
    }
}

impl Modal {
    pub fn new(cx: &mut WindowContext) -> Self {
        let base = v_flex()
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .shadow_xl()
            .min_h_48()
            .p_4()
            .gap_4();

        Self {
            base,
            title: None,
            footer: None,
            content: v_flex(),
            margin_top: None,
            width: px(480.),
            max_width: None,
            overlay: true,
            on_close: Rc::new(|_, _| {}),
            show_close: true,
        }
    }

    /// Sets the title of the modal.
    pub fn title(mut self, title: impl IntoElement) -> Self {
        self.title = Some(title.into_any_element());
        self
    }

    /// Set the footer of the modal.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Sets the callback for when the modal is closed.
    pub fn on_close(
        mut self,
        on_close: impl Fn(&ClickEvent, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_close = Rc::new(on_close);
        self
    }

    /// Sets the false to hide close icon, default: true
    pub fn show_close(mut self, show_close: bool) -> Self {
        self.show_close = show_close;
        self
    }

    /// Set the top offset of the modal, defaults to None, will use the 1/10 of the viewport height.
    pub fn margin_top(mut self, margin_top: Pixels) -> Self {
        self.margin_top = Some(margin_top);
        self
    }

    /// Sets the width of the modal, defaults to 480px.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// Set the maximum width of the modal, defaults to `None`.
    pub fn max_width(mut self, max_width: Pixels) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Set the overlay of the modal, defaults to `true`.
    pub fn overlay(mut self, overlay: bool) -> Self {
        self.overlay = overlay;
        self
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements);
    }
}

impl Styled for Modal {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Modal {
    fn render(self, cx: &mut WindowContext) -> impl gpui::IntoElement {
        let on_close = self.on_close.clone();
        let view_size = cx.viewport_size();
        let bounds = Bounds {
            origin: Point::default(),
            size: view_size,
        };
        let y = self.margin_top.unwrap_or(view_size.height / 10.);
        let x = bounds.center().x - self.width / 2.;

        anchored().snap_to_window().child(
            div()
                .occlude()
                .w(view_size.width)
                .h(view_size.height)
                .bg(overlay_color(self.overlay, cx))
                .when(self.overlay, |this| {
                    this.on_mouse_down(MouseButton::Left, {
                        let on_close = self.on_close.clone();
                        move |_, cx| {
                            on_close(&ClickEvent::default(), cx);
                            cx.close_modal();
                        }
                    })
                })
                .child(
                    self.base
                        .id("modal")
                        .absolute()
                        .occlude()
                        .relative()
                        .left(x)
                        .top(y)
                        .w(self.width)
                        .when_some(self.max_width, |this, w| this.max_w(w))
                        .children(self.title)
                        .when(self.show_close, |this| {
                            this.child(
                                Button::new("close", cx)
                                    .absolute()
                                    .top_2()
                                    .right_2()
                                    .small()
                                    .ghost()
                                    .icon(IconName::Close)
                                    .on_click(move |_, cx| {
                                        on_close(&ClickEvent::default(), cx);
                                        cx.close_modal();
                                    }),
                            )
                        })
                        .child(self.content)
                        .children(self.footer)
                        .with_animation(
                            "slide-down",
                            Animation::new(Duration::from_secs_f64(0.1)),
                            move |this, delta| {
                                let y_offset = px(-30.) + delta * px(30.);
                                this.top(y + y_offset)
                            },
                        ),
                ),
        )
    }
}
