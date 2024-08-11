use crate::h_flex;
use crate::theme::ActiveTheme;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, AnyElement, Div, IntoElement, ParentElement, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled, WindowContext,
};
use gpui::{px, InteractiveElement};
use smallvec::SmallVec;

#[derive(IntoElement)]
pub struct TabBar {
    base: Div,
    id: SharedString,
    scroll_handle: ScrollHandle,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    children: SmallVec<[AnyElement; 2]>,
}

impl TabBar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            base: div().h_8().px(px(-1.)),
            id: id.into(),
            children: SmallVec::new(),
            scroll_handle: ScrollHandle::new(),
            prefix: None,
            suffix: None,
        }
    }

    #[allow(unused)]
    pub fn track_scroll(mut self, scroll_handle: ScrollHandle) -> Self {
        self.scroll_handle = scroll_handle;
        self
    }

    /// Set the prefix element of the TabBar
    pub fn prefix(mut self, prefix: impl Into<AnyElement>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the suffix element of the TabBar
    pub fn suffix(mut self, suffix: impl Into<AnyElement>) -> Self {
        self.suffix = Some(suffix.into());
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
            .bg(theme.tab_bar)
            .border_b_1()
            .border_color(cx.theme().border)
            .text_color(theme.tab_foreground)
            .when_some(self.prefix, |this, prefix| this.child(prefix))
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
            .when_some(self.suffix, |this, suffix| this.child(suffix))
    }
}
