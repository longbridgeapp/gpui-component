use std::{
    cell::{Cell, RefCell},
    ops::Range,
    rc::Rc,
};

use crate::theme::ActiveTheme;
use gpui::{
    deferred, div, fill, point, px, relative, AnyElement, AnyView, Bounds, ContentMask, Element,
    Hitbox, InteractiveElement, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ScrollWheelEvent, StatefulInteractiveElement as _, Style, Styled as _, UniformListScrollHandle,
};

pub struct Scrollbar {
    items_count: usize,
    width: f32,
    list_height: f64,
    current_offset: f64,
    thumb: Range<f32>,
    handle: UniformListScrollHandle,
    drag_state: Rc<Cell<Option<f32>>>,
    view: AnyView,
}

const MIN_THUMB_PERCENTAGE_HEIGHT: f64 = 0.03;

impl Scrollbar {
    pub fn new(
        view: AnyView,
        handle: UniformListScrollHandle,
        items_count: usize,
        show_scrollbar: bool,
    ) -> Option<Self> {
        let cloned_handle = handle.clone();

        // Ref from: Zed crates\project_panel\src\project_panel.rs
        // render_scrollbar method
        let scroll_state = handle.0.borrow();
        let last_item_height = scroll_state
            .last_item_height
            .filter(|_| show_scrollbar && items_count > 0)?;
        let list_height = items_count as f64 * last_item_height.0 as f64;
        let current_offset = scroll_state.base_handle.offset().y.0.min(0.).abs() as f64;
        let mut percentage = current_offset / list_height;
        let end_offset: f64 =
            (current_offset + scroll_state.base_handle.bounds().size.height.0 as f64) / list_height;
        let overshoot = (end_offset - 1.).clamp(0., 1.);
        if overshoot > 0. {
            percentage -= overshoot;
        }

        if percentage + MIN_THUMB_PERCENTAGE_HEIGHT > 1.0 || end_offset > list_height {
            return None;
        }
        if list_height < scroll_state.base_handle.bounds().size.height.0 as f64 {
            return None;
        }

        let end_offset = end_offset.clamp(percentage + MIN_THUMB_PERCENTAGE_HEIGHT, 1.);

        let thumb = percentage as f32..end_offset as f32;

        Some(Self {
            view,
            items_count,
            list_height,
            width: 10.0,
            thumb,
            handle: cloned_handle,
            current_offset,
            drag_state: Rc::new(Cell::new(None)),
        })
    }

    fn render_bar(&self, cx: &mut gpui::WindowContext) -> impl IntoElement {
        div()
            .id("scrollbar")
            .occlude()
            .absolute()
            .right_0()
            .top_0()
            .bottom_0()
            .h_full()
            .w(px(self.width))
            .cursor_default()
    }
}

impl IntoElement for Scrollbar {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Scrollbar {
    type RequestLayoutState = ();

    type PrepaintState = Hitbox;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 0.0;
        style.flex_shrink = 1.;
        style.size.width = px(self.width).into();
        style.size.height = relative(1.).into();

        let mut bar = deferred(self.render_bar(cx).into_any_element())
            .priority(1)
            .into_any_element();
        let bar_layout_id = bar.request_layout(cx);

        (cx.request_layout(style, vec![bar_layout_id]), ())
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            cx.insert_hitbox(bounds, false)
        })
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            let bar_bg = cx.theme().scrollbar;
            let thumb_bg = cx.theme().scrollbar_thumb;

            let thumb_offset = self.thumb.start * bounds.size.height;
            let thumb_end = self.thumb.end * bounds.size.height;
            let thumb_percentage_size = self.thumb.end - self.thumb.start;
            let thumb_bounds = {
                let thumb_upper_left = point(bounds.origin.x, bounds.origin.y + thumb_offset);
                let thumb_lower_right = point(
                    bounds.origin.x + bounds.size.width,
                    bounds.origin.y + thumb_end,
                );
                Bounds::from_corners(thumb_upper_left, thumb_lower_right)
            };

            cx.paint_quad(fill(bounds, bar_bg));
            cx.paint_quad(fill(thumb_bounds, thumb_bg));

            let handle = self.handle.clone();
            let items_count = self.items_count;

            cx.on_mouse_event({
                let scroll = self.handle.clone();
                let is_dragging = self.drag_state.clone();
                move |event: &MouseDownEvent, phase, _cx| {
                    if phase.bubble() && bounds.contains(&event.position) {
                        if !thumb_bounds.contains(&event.position) {
                            let scroll = scroll.0.borrow();
                            if let Some(last_height) = scroll.last_item_height {
                                let max_offset = items_count as f32 * last_height;
                                let percentage =
                                    (event.position.y - bounds.origin.y) / bounds.size.height;

                                let percentage = percentage.min(1. - thumb_percentage_size);
                                scroll
                                    .base_handle
                                    .set_offset(point(px(0.), -max_offset * percentage));
                            }
                        } else {
                            let thumb_top_offset =
                                (event.position.y - thumb_bounds.origin.y) / bounds.size.height;
                            is_dragging.set(Some(thumb_top_offset));
                        }
                    }
                }
            });
            cx.on_mouse_event({
                let scroll = self.handle.clone();
                move |event: &ScrollWheelEvent, phase, cx| {
                    if phase.bubble() && bounds.contains(&event.position) {
                        let scroll = scroll.0.borrow_mut();
                        let current_offset = scroll.base_handle.offset();
                        scroll
                            .base_handle
                            .set_offset(current_offset + event.delta.pixel_delta(cx.line_height()));
                    }
                }
            });
            let drag_state = self.drag_state.clone();
            let view_id = self.view.entity_id();
            cx.on_mouse_event(move |event: &MouseMoveEvent, _, cx| {
                if let Some(drag_state) = drag_state.get().filter(|_| event.dragging()) {
                    let scroll = handle.0.borrow();
                    if let Some(last_height) = scroll.last_item_height {
                        let max_offset = items_count as f32 * last_height;
                        let percentage =
                            (event.position.y - bounds.origin.y) / bounds.size.height - drag_state;

                        let percentage = percentage.min(1. - thumb_percentage_size);
                        scroll
                            .base_handle
                            .set_offset(point(px(0.), -max_offset * percentage));
                        cx.notify(view_id);
                    }
                } else {
                    drag_state.set(None);
                }
            });
            let is_dragging = self.drag_state.clone();
            cx.on_mouse_event(move |_event: &MouseUpEvent, phase, cx| {
                if phase.bubble() {
                    is_dragging.set(None);
                    cx.notify(view_id);
                }
            });
        })
    }
}
