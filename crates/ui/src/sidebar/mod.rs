use crate::{theme::ActiveTheme, v_flex, Side};
use gpui::{
    div, prelude::FluentBuilder, px, AnyElement, Div, Empty, InteractiveElement as _, IntoElement,
    ParentElement, Pixels, Render, RenderOnce, SharedString, Styled as _, ViewContext,
    WindowContext,
};
use std::rc::Rc;

mod menu;
pub use menu::*;

const DEFAULT_WIDTH: Pixels = px(255.);

/// A sidebar
pub struct Sidebar {
    content: Rc<dyn Fn(&mut WindowContext) -> AnyElement>,
    /// header view
    header: Option<Rc<dyn Fn(&mut WindowContext) -> AnyElement>>,
    /// footer view
    footer: Option<Rc<dyn Fn(&mut WindowContext) -> AnyElement>>,
    /// The side of the sidebar
    side: Side,
    collapsible: bool,
    width: Pixels,
    is_collapsed: bool,
}

impl Sidebar {
    pub fn left() -> Self {
        Self {
            content: Rc::new(|_| Empty.into_any_element()),
            header: None,
            footer: None,
            side: Side::Left,
            collapsible: true,
            width: DEFAULT_WIDTH,
            is_collapsed: false,
        }
    }

    pub fn right() -> Self {
        Self {
            content: Rc::new(|_| Empty.into_any_element()),
            header: None,
            footer: None,
            side: Side::Right,
            collapsible: true,
            width: DEFAULT_WIDTH,
            is_collapsed: false,
        }
    }

    /// Set the width of the sidebar
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// Set the sidebar to be collapsible, default is true
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Set the sidebar to be collapsed
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut ViewContext<Self>) {
        self.is_collapsed = collapsed;
        cx.notify();
    }

    pub fn header<F, E>(mut self, header: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut WindowContext) -> E + 'static,
    {
        self.header = Some(Rc::new(move |cx| header(cx).into_any_element()));
        self
    }

    pub fn footer<F, E>(mut self, footer: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut WindowContext) -> E + 'static,
    {
        self.footer = Some(Rc::new(move |cx| footer(cx).into_any_element()));
        self
    }

    pub fn content<F, E>(mut self, content: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut WindowContext) -> E + 'static,
    {
        self.content = Rc::new(move |cx| content(cx).into_any_element());
        self
    }
}

/// A sidebar group
#[derive(IntoElement)]
pub struct SidebarGroup {
    base: Div,
    label: SharedString,
}

impl SidebarGroup {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            base: div().gap_2().flex_col(),
            label: label.into(),
        }
    }
}

impl RenderOnce for SidebarGroup {
    fn render(self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        v_flex()
            .relative()
            .p_2()
            .child(
                div()
                    .flex_shrink_0()
                    .px_2()
                    .rounded_md()
                    .text_xs()
                    .text_color(cx.theme().sidebar_foreground)
                    .h_8()
                    .child(self.label),
            )
            .child(self.base)
    }
}

impl Render for Sidebar {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        v_flex()
            .id("sidebar")
            .w(self.width)
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_color(cx.theme().sidebar_border)
            .map(|this| match self.side {
                Side::Left => this.border_r_1(),
                Side::Right => this.text_2xl(),
            })
            .when_some(self.header.clone(), |this, header| this.child(header(cx)))
            .child(
                v_flex()
                    .flex_1()
                    .id("sidebar-content")
                    .gap_2()
                    .child((self.content)(cx)),
            )
            .when_some(self.footer.clone(), |this, footer| this.child(footer(cx)))
    }
}
