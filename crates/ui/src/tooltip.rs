use gpui::{
    div, prelude::FluentBuilder, px, AnyView, Div, IntoElement, ParentElement, Render,
    SharedString, Styled, ViewContext, VisualContext, WindowContext,
};

use crate::{h_flex, styled_ext::ElevationIndex, theme::ActiveTheme, v_flex};

pub struct Tooltip {
    title: SharedString,
    meta: Option<SharedString>,
}

impl Tooltip {
    pub fn text(title: impl Into<SharedString>, cx: &mut WindowContext) -> AnyView {
        cx.new_view(|_cx| Self {
            title: title.into(),
            meta: None,
        })
        .into()
    }

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            meta: None,
        }
    }

    pub fn with_meta(
        title: impl Into<SharedString>,
        meta: impl Into<SharedString>,
        cx: &mut WindowContext,
    ) -> AnyView {
        cx.new_view(|_: &mut ViewContext<Tooltip>| Self {
            title: title.into(),
            meta: Some(meta.into()),
        })
        .into()
    }

    // TODO: pub fn for_action
}

impl Render for Tooltip {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        tooltip_container(cx, |el, _| {
            el.child(h_flex().gap_4().child(self.title.clone()))
                .when_some(self.meta.clone(), |this, meta| {
                    this.child(div().text_size(px(12.)).child(meta))
                })
        })
    }
}

pub fn tooltip_container<V>(
    cx: &mut ViewContext<V>,
    f: impl FnOnce(Div, &mut ViewContext<V>) -> Div,
) -> impl IntoElement {
    // padding to avoid tooltip appearing right below the mouse cursor
    div().pl_2().pt_2p5().child(
        v_flex()
            .bg(cx.theme().popover)
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border)
            .shadow(ElevationIndex::ElevatedSurface.shadow())
            .text_color(cx.theme().popover_foreground)
            .py_1p5()
            .px_2()
            .map(|el| f(el, cx)),
    )
}
