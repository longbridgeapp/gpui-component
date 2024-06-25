use crate::stock::h_flex;
use crate::theme::ActiveTheme;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, AnyElement, Div, IntoElement, ParentElement, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, WindowContext,
};
use gpui::{InteractiveElement, Styled as _};
use smallvec::SmallVec;

#[derive(IntoElement)]
pub struct TabBar {
    base: Div,
    id: SharedString,
    scroll_handle: Option<ScrollHandle>,
    children: SmallVec<[AnyElement; 2]>,
}

impl TabBar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            id: id.into(),
            children: SmallVec::new(),
            scroll_handle: None,
        }
    }

    #[allow(unused)]
    pub fn track_scroll(mut self, scroll_handle: ScrollHandle) -> Self {
        self.scroll_handle = Some(scroll_handle);
        self
    }
}

impl ParentElement for TabBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for TabBar {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();

        self.base
            .id(self.id)
            .group("tab_bar")
            .flex()
            .flex_none()
            .items_center()
            .w_full()
            .h_10()
            .p_1()
            .bg(theme.muted)
            .text_color(theme.muted_foreground)
            .rounded_md()
            .relative()
            .overflow_x_hidden()
            .child(
                h_flex()
                    .id("tabs")
                    .flex_grow()
                    .overflow_x_scroll()
                    .when_some(self.scroll_handle, |cx, scroll_handle| {
                        cx.track_scroll(&scroll_handle)
                    })
                    .children(self.children),
            )
    }
}
