use std::{cell::Cell, ops::Range, rc::Rc};

use crate::theme::{ActiveTheme, Colorize};
use gpui::{
    fill, point, px, relative, AnyView, Bounds, ContentMask, Element, Hitbox, IntoElement,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, ScrollWheelEvent, Style,
    UniformListScrollHandle,
};

const MIN_THUMB_PERCENTAGE_HEIGHT: f64 = 0.03;
const THUMB_RADIUS: Pixels = Pixels(5.0);
const THUMB_INSET: Pixels = Pixels(0.8);

pub struct Scrollbar {
    width: f32,
    view: AnyView,
    handle: UniformListScrollHandle,
    /// This is the state of the scrollbar thumb when it is being dragged.
    /// It must ref from the parent view.
    drag_state: Rc<Cell<Option<f32>>>,
    items_count: usize,
    thumb: Range<f32>,
}

impl Scrollbar {
    pub fn new(
        view: AnyView,
        handle: UniformListScrollHandle,
        drag_state: Rc<Cell<Option<f32>>>,
        items_count: usize,
        show_scrollbar: bool,
    ) -> Option<Self> {
        let cloned_handle = handle.clone();

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
            width: 12.0,
            thumb,
            handle: cloned_handle,
            drag_state,
        })
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}

impl IntoElement for Scrollbar {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct ScrollbarState {}

impl Element for Scrollbar {
    type RequestLayoutState = ScrollbarState;

    type PrepaintState = Hitbox;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.flex_grow = 0.0;
        style.flex_shrink = 1.;
        style.size.width = px(self.width).into();
        style.size.height = relative(1.).into();

        (cx.request_layout(style, None), ScrollbarState {})
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            cx.insert_hitbox(bounds, false)
        })
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            let is_draging = self.drag_state.get().is_some();
            let bar_bg = cx.theme().scrollbar;
            let thumb_bg = cx.theme().scrollbar_thumb;
            let thumb_bg = if is_draging {
                thumb_bg.darken(0.1)
            } else {
                thumb_bg
            };

            let thumb_top = self.thumb.start * bounds.size.height;
            let thumb_bottom = self.thumb.end * bounds.size.height;
            let thumb_percentage_size = self.thumb.end - self.thumb.start;
            let thumb_bounds = Bounds::from_corners(
                point(
                    bounds.origin.x + THUMB_INSET,
                    bounds.origin.y + thumb_top + THUMB_INSET,
                ),
                point(
                    bounds.origin.x + bounds.size.width - (THUMB_INSET * 2),
                    bounds.origin.y + thumb_bottom - (THUMB_INSET * 2),
                ),
            );

            cx.paint_quad(fill(bounds, bar_bg));
            cx.paint_quad(fill(thumb_bounds, thumb_bg).corner_radii(THUMB_RADIUS));

            let handle = self.handle.clone();
            let items_count = self.items_count;

            let drag_state = self.drag_state.clone();
            cx.on_mouse_event({
                let scroll = self.handle.clone();
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
                            drag_state.set(Some(thumb_top_offset));
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

            let view_id = self.view.entity_id();
            let drag_state = self.drag_state.clone();
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

            let drag_state = self.drag_state.clone();
            cx.on_mouse_event(move |_event: &MouseUpEvent, phase, cx| {
                if phase.bubble() {
                    drag_state.set(None);
                    cx.notify(view_id);
                }
            });
        })
    }
}
