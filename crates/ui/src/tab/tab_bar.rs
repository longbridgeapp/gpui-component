use crate::stock::h_flex;
use crate::theme::ActiveTheme;
use crate::StyledExt;
use gpui::prelude::FluentBuilder as _;
use gpui::InteractiveElement;
use gpui::{
    div, AnyElement, Div, IntoElement, ParentElement, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled, WindowContext,
};
use smallvec::SmallVec;

#[derive(IntoElement)]
pub struct TabBar {
    base: Div,
    id: SharedString,
    scroll_handle: ScrollHandle,
    children: SmallVec<[AnyElement; 2]>,
}

impl TabBar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            base: div().h_10().p_1(),
            id: id.into(),
            children: SmallVec::new(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    #[allow(unused)]
    pub fn track_scroll(mut self, scroll_handle: ScrollHandle) -> Self {
        self.scroll_handle = scroll_handle;
        self
    }
}

impl ParentElement for TabBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl Styled for TabBar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for TabBar {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.theme();

        self.base
            .id(self.id)
            .group("tab-bar")
            .flex()
            .flex_none()
            .items_center()
            .bg(theme.muted)
            .text_color(theme.muted_foreground)
            // The child will append to this level
            .child(
                h_flex()
                    .id("tabs")
                    .flex_grow()
                    .overflow_x_scroll()
                    .track_scroll(&self.scroll_handle)
                    // The children will append to this level
                    .children(self.children),
            )
    }
}
