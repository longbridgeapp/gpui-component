use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    canvas, div, px, Bounds, InteractiveElement, ParentElement, Pixels, Render, ScrollHandle,
    StatefulInteractiveElement as _, Styled, View, VisualContext, WindowContext,
};
use ui::new_scrollbar::{Scrollbar, ScrollbarState};
use ui::theme::ActiveTheme;
use ui::{v_flex, StyledExt};

pub struct ScrollableStory {
    scroll_handle: ScrollHandle,
    scroll_size: gpui::Size<Pixels>,
    scroll_state: Rc<Cell<ScrollbarState>>,
    items: Vec<String>,
}

impl ScrollableStory {
    fn new() -> Self {
        let items = (0..500).map(|i| format!("Item {}", i)).collect();
        println!("ScrollableStory");
        Self {
            scroll_handle: ScrollHandle::new(),
            scroll_state: Rc::new(Cell::new(ScrollbarState::default())),
            scroll_size: gpui::Size::default(),
            items,
        }
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self::new())
    }
}

impl Render for ScrollableStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let view = cx.view().clone();

        div()
            .relative()
            .w_full()
            .h(px(400.))
            .mb_10()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .id("scroll-story")
                    .overflow_scroll()
                    .p_1()
                    .size_full()
                    .track_scroll(&self.scroll_handle)
                    .child(
                        v_flex()
                            .gap_1()
                            .w(px(2540.))
                            .children(
                                self.items
                                    .iter()
                                    .map(|s| div().debug_green().child(s.clone())),
                            )
                            .child({
                                let view = cx.view().clone();
                                canvas(
                                    move |bounds, cx| {
                                        view.update(cx, |r, _| r.scroll_size = bounds.size)
                                    },
                                    |_, _, _| {},
                                )
                                .absolute()
                                .size_full()
                            }),
                    ),
            )
            .child(Scrollbar::both(
                view,
                self.scroll_state.clone(),
                self.scroll_handle.clone(),
                self.scroll_size,
            ))
    }
}
