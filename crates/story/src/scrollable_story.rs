use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    canvas, deferred, div, px, InteractiveElement, ParentElement, Pixels, Render, ScrollHandle,
    StatefulInteractiveElement as _, Styled, View, ViewContext, VisualContext, WindowContext,
};
use ui::button::Button;
use ui::divider::Divider;
use ui::scroll::{Scrollbar, ScrollbarAxis, ScrollbarState};
use ui::theme::ActiveTheme;
use ui::{h_flex, v_flex, Clickable};

pub struct ScrollableStory {
    scroll_handle: ScrollHandle,
    scroll_size: gpui::Size<Pixels>,
    scroll_state: Rc<Cell<ScrollbarState>>,
    items: Vec<String>,
    test_width: Pixels,
    axis: ScrollbarAxis,
}

impl ScrollableStory {
    fn new() -> Self {
        Self {
            scroll_handle: ScrollHandle::new(),
            scroll_state: Rc::new(Cell::new(ScrollbarState::default())),
            scroll_size: gpui::Size::default(),
            items: (0..500).map(|i| format!("Item {}", i)).collect::<Vec<_>>(),
            test_width: px(3000.),
            axis: ScrollbarAxis::Both,
        }
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self::new())
    }

    pub fn change_test_cases(&mut self, n: usize, cx: &mut ViewContext<Self>) {
        if n == 0 {
            self.items = (0..500).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
            self.test_width = px(3000.);
        } else if n == 1 {
            self.items = (0..100).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
            self.test_width = px(10000.);
        } else {
            self.items = (0..500).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
            self.test_width = px(10000.);
        }
        self.scroll_state.set(ScrollbarState::default());
        cx.notify();
    }

    pub fn change_axis(&mut self, axis: ScrollbarAxis, cx: &mut ViewContext<Self>) {
        self.axis = axis;
        cx.notify();
    }
}

impl Render for ScrollableStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let view = cx.view().clone();

        v_flex()
            .gap_4()
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("test-0", cx)
                            .label("Size 0")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(0, cx);
                            })),
                    )
                    .child(
                        Button::new("test-1", cx)
                            .label("Size 1")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(1, cx);
                            })),
                    )
                    .child(
                        Button::new("test-2", cx)
                            .label("Size 2")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(2, cx);
                            })),
                    )
                    .child(Divider::vertical())
                    .child(
                        Button::new("test-axis-both", cx)
                            .label("Both Scrollbar")
                            .on_click(
                                cx.listener(|view, _, cx| {
                                    view.change_axis(ScrollbarAxis::Both, cx)
                                }),
                            ),
                    )
                    .child(
                        Button::new("test-axis-vertical", cx)
                            .label("Vertical")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_axis(ScrollbarAxis::Vertical, cx)
                            })),
                    )
                    .child(
                        Button::new("test-axis-both", cx)
                            .label("Horizontal")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_axis(ScrollbarAxis::Horizontal, cx)
                            })),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .relative()
                            .w_full()
                            .h(px(400.))
                            .child(deferred(
                                Scrollbar::both(
                                    view,
                                    self.scroll_state.clone(),
                                    self.scroll_handle.clone(),
                                    self.scroll_size,
                                )
                                .axis(self.axis),
                            ))
                            .child(
                                div()
                                    .id("scroll-story")
                                    .overflow_scroll()
                                    .p_4()
                                    .size_full()
                                    .track_scroll(&self.scroll_handle)
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .w(self.test_width)
                                            .children(self.items.iter().map(|s| {
                                                div().bg(cx.theme().card).child(s.clone())
                                            }))
                                            .child({
                                                let view = cx.view().clone();
                                                canvas(
                                                    move |bounds, cx| {
                                                        view.update(cx, |r, _| {
                                                            r.scroll_size = bounds.size
                                                        })
                                                    },
                                                    |_, _, _| {},
                                                )
                                                .absolute()
                                                .size_full()
                                            }),
                                    ),
                            ),
                    ),
            )
    }
}
