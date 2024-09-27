use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    canvas, div, px, Entity, InteractiveElement, ParentElement, Pixels, Render, ScrollHandle,
    StatefulInteractiveElement as _, Styled, View, ViewContext, VisualContext, WindowContext,
};
use ui::button::Button;
use ui::divider::Divider;
use ui::scroll::{Scrollbar, ScrollbarAxis, ScrollbarState};
use ui::theme::ActiveTheme;
use ui::{h_flex, v_flex, StyledExt as _};

pub struct ScrollableStory {
    focus_handle: gpui::FocusHandle,
    scroll_handle: ScrollHandle,
    scroll_size: gpui::Size<Pixels>,
    scroll_state: Rc<Cell<ScrollbarState>>,
    items: Vec<String>,
    test_width: Pixels,
    axis: ScrollbarAxis,
}

impl ScrollableStory {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            scroll_state: Rc::new(Cell::new(ScrollbarState::default())),
            scroll_size: gpui::Size::default(),
            items: (0..500).map(|i| format!("Item {}", i)).collect::<Vec<_>>(),
            test_width: px(3000.),
            axis: ScrollbarAxis::Both,
        }
    }

    pub fn view(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(Self::new)
    }

    pub fn change_test_cases(&mut self, n: usize, cx: &mut ViewContext<Self>) {
        if n == 0 {
            self.items = (0..500).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
            self.test_width = px(3000.);
        } else if n == 1 {
            self.items = (0..100).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
            self.test_width = px(10000.);
        } else if n == 2 {
            self.items = (0..500).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
            self.test_width = px(10000.);
        } else {
            self.items = (0..5).map(|i| format!("Item {}", i)).collect::<Vec<_>>();
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

impl super::Story for ScrollableStory {
    fn title() -> &'static str {
        "Scrollable"
    }

    fn description() -> &'static str {
        "Add vertical or horizontal, or both scrollbars to a container."
    }

    fn new_view(cx: &mut WindowContext) -> View<impl gpui::FocusableView> {
        Self::view(cx)
    }
}

impl gpui::FocusableView for ScrollableStory {
    fn focus_handle(&self, _: &gpui::AppContext) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ScrollableStory {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let view = cx.view().clone();

        v_flex()
            .size_full()
            .gap_4()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("test-0")
                            .label("Size 0")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(0, cx);
                            })),
                    )
                    .child(
                        Button::new("test-1")
                            .label("Size 1")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(1, cx);
                            })),
                    )
                    .child(
                        Button::new("test-2")
                            .label("Size 2")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(2, cx);
                            })),
                    )
                    .child(
                        Button::new("test-3")
                            .label("Size 3")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_test_cases(3, cx);
                            })),
                    )
                    .child(Divider::vertical().px_2())
                    .child(
                        Button::new("test-axis-both")
                            .label("Both Scrollbar")
                            .on_click(
                                cx.listener(|view, _, cx| {
                                    view.change_axis(ScrollbarAxis::Both, cx)
                                }),
                            ),
                    )
                    .child(
                        Button::new("test-axis-vertical")
                            .label("Vertical")
                            .on_click(cx.listener(|view, _, cx| {
                                view.change_axis(ScrollbarAxis::Vertical, cx)
                            })),
                    )
                    .child(
                        Button::new("test-axis-horizontal")
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
                            .h(px(350.))
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
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .right_0()
                                    .bottom_0()
                                    .child(
                                        Scrollbar::both(
                                            view.entity_id(),
                                            self.scroll_state.clone(),
                                            self.scroll_handle.clone(),
                                            self.scroll_size,
                                        )
                                        .axis(self.axis),
                                    ),
                            ),
                    ),
            )
            .child({
                let items = self.items.clone();
                let test_width = self.test_width;

                div()
                    .relative()
                    .border_1()
                    .border_color(cx.theme().border)
                    .w_full()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .id("test-1")
                            .scrollable(cx.view().entity_id(), ScrollbarAxis::Vertical)
                            .focusable()
                            .p_3()
                            .w(test_width)
                            .gap_1()
                            .child("Hello world")
                            .children(
                                items
                                    .iter()
                                    .map(|s| div().bg(cx.theme().card).child(s.clone())),
                            ),
                    )
            })
    }
}
