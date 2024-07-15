use core::arch;
use std::{borrow::BorrowMut, cell::Cell, ops::Range, rc::Rc};

use crate::{
    red_200,
    theme::{ActiveTheme, Colorize},
};
use gpui::{
    fill, point, px, relative, AnyView, Bounds, ContentMask, Element, Hitbox, IntoElement,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Position, ScrollHandle, Style,
};

const MIN_THUMB_SIZE: f32 = 80.;
const THUMB_RADIUS: Pixels = Pixels(5.0);
const THUMB_INSET: Pixels = Pixels(0.8);

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarState {
    hovered_axis: Option<ScrollbarAxis>,
    draged_axis: Option<ScrollbarAxis>,
    drag_pos: Point<Pixels>,
    visible: bool,
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self {
            hovered_axis: None,
            draged_axis: None,
            drag_pos: point(px(0.), px(0.)),
            visible: true,
        }
    }
}

impl ScrollbarState {
    fn set_drag_pos(&self, axis: ScrollbarAxis, pos: Point<Pixels>) -> Self {
        let mut state = *self;
        if axis.is_vertical() {
            state.drag_pos.y = pos.y;
        } else {
            state.drag_pos.x = pos.x;
        }

        state.draged_axis = Some(axis);
        state
    }

    fn unset_drag_pos(&self) -> Self {
        let mut state = *self;
        state.draged_axis = None;
        state
    }

    fn set_hovered(&self, axis: Option<ScrollbarAxis>) -> Self {
        let mut state = *self;
        state.hovered_axis = axis;
        state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
    Vertical,
    Horizontal,
    Both,
}

impl ScrollbarAxis {
    fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    fn is_both(&self) -> bool {
        matches!(self, Self::Both)
    }

    pub fn has_vertical(&self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }

    pub fn has_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    fn all(&self) -> Vec<ScrollbarAxis> {
        match self {
            Self::Vertical => vec![Self::Vertical],
            Self::Horizontal => vec![Self::Horizontal],
            Self::Both => vec![Self::Vertical, Self::Horizontal],
        }
    }
}

pub struct Scrollbar {
    view: AnyView,
    axis: ScrollbarAxis,
    /// When is vertical, this is the height of the scrollbar.
    width: Pixels,
    scroll_handle: ScrollHandle,
    scroll_size: gpui::Size<Pixels>,
    state: Rc<Cell<ScrollbarState>>,
}

impl Scrollbar {
    fn new(
        view: AnyView,
        state: Rc<Cell<ScrollbarState>>,
        axis: ScrollbarAxis,
        scroll_handle: ScrollHandle,
        scroll_size: gpui::Size<Pixels>,
    ) -> Self {
        Self {
            view,
            state,
            axis,
            scroll_size,
            width: px(12.),
            scroll_handle,
        }
    }

    pub fn both(
        view: impl Into<AnyView>,
        state: Rc<Cell<ScrollbarState>>,
        scroll_handle: ScrollHandle,
        scroll_size: gpui::Size<Pixels>,
    ) -> Self {
        Self::new(
            view.into(),
            state,
            ScrollbarAxis::Both,
            scroll_handle,
            scroll_size,
        )
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
        _: Option<&gpui::GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.position = Position::Absolute;
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();

        (cx.request_layout(style, None), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        // Move up to cover the parent bounds.
        let bounds = Bounds {
            origin: point(bounds.origin.x, bounds.origin.y - bounds.size.height),
            size: bounds.size,
        };

        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            cx.insert_hitbox(bounds, false)
        })
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        cx: &mut gpui::WindowContext,
    ) {
        let bounds = hitbox.bounds;
        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            for axis in self.axis.all().into_iter() {
                let (scroll_area_size, container_size, current_offset) = if axis.is_vertical() {
                    (
                        self.scroll_size.height,
                        bounds.size.height,
                        self.scroll_handle.offset().y,
                    )
                } else {
                    (
                        self.scroll_size.width,
                        bounds.size.width
                            - if self.axis.is_both() {
                                self.width
                            } else {
                                px(0.)
                            },
                        self.scroll_handle.offset().x,
                    )
                };

                let thumb_size_ratio = (container_size / scroll_area_size).clamp(0.0, 1.0);

                let thumb_size = (container_size.0 * thumb_size_ratio).max(MIN_THUMB_SIZE);
                let thumb_start = px(current_offset / (-scroll_area_size)) * container_size;
                let thumb_end = thumb_start + px(thumb_size);

                let bounds = Bounds {
                    origin: if axis.is_vertical() {
                        point(
                            bounds.origin.x + bounds.size.width - self.width,
                            bounds.origin.y,
                        )
                    } else {
                        point(
                            bounds.origin.x,
                            bounds.origin.y + bounds.size.height - self.width,
                        )
                    },
                    size: gpui::Size {
                        width: if axis.is_vertical() {
                            self.width
                        } else {
                            bounds.size.width
                        },
                        height: if axis.is_vertical() {
                            bounds.size.height
                        } else {
                            self.width
                        },
                    },
                };

                let bar_bg = cx.theme().scrollbar;
                let thumb_bg = cx.theme().scrollbar_thumb;
                let state = self.state.clone();
                let (thumb_bg, inset) = if state.get().draged_axis == Some(axis) {
                    (thumb_bg.darken(0.3), THUMB_INSET)
                } else if state.get().hovered_axis == Some(axis) {
                    (thumb_bg.darken(0.15), THUMB_INSET)
                } else {
                    (thumb_bg, THUMB_INSET)
                };

                let thumb_bounds = if axis.is_vertical() {
                    Bounds::from_corners(
                        point(
                            bounds.origin.x + inset,
                            bounds.origin.y + thumb_start + inset,
                        ),
                        point(
                            bounds.origin.x + self.width - (inset * 2),
                            bounds.origin.y + thumb_end - (inset * 2),
                        ),
                    )
                } else {
                    Bounds::from_corners(
                        point(
                            bounds.origin.x + thumb_start * inset,
                            bounds.origin.y + inset,
                        ),
                        point(
                            bounds.origin.x + thumb_end - (inset * 2),
                            bounds.origin.y + self.width - (inset * 2),
                        ),
                    )
                };

                cx.paint_quad(fill(bounds, bar_bg));
                cx.paint_quad(fill(thumb_bounds, thumb_bg).corner_radii(THUMB_RADIUS));

                cx.on_mouse_event({
                    let state = self.state.clone();
                    let view_id = self.view.entity_id();

                    move |event: &MouseDownEvent, phase, cx| {
                        if phase.bubble() && thumb_bounds.contains(&event.position) {
                            let drag_pos = if axis.is_vertical() {
                                point(event.position.x, event.position.y - thumb_bounds.origin.y)
                            } else {
                                point(event.position.x - thumb_bounds.origin.x, event.position.y)
                            };

                            state.set(state.get().set_drag_pos(axis, drag_pos));
                            cx.notify(view_id);
                        }
                    }
                });

                cx.on_mouse_event({
                    let scroll_handle = self.scroll_handle.clone();
                    let state = self.state.clone();
                    let view_id = self.view.entity_id();

                    move |event: &MouseMoveEvent, _, cx| {
                        if thumb_bounds.contains(&event.position) {
                            if state.get().hovered_axis != Some(axis) {
                                state.set(state.get().set_hovered(Some(axis)));
                                cx.notify(view_id);
                            }
                        } else {
                            if state.get().hovered_axis == Some(axis) {
                                if state.get().hovered_axis.is_some() {
                                    state.set(state.get().set_hovered(None));
                                    cx.notify(view_id);
                                }
                            }
                        }

                        if event.dragging() && state.get().draged_axis == Some(axis) {
                            let drag_pos = state.get().drag_pos;
                            let percentage = if axis.is_vertical() {
                                (event.position.y - bounds.origin.y - drag_pos.y)
                                    / bounds.size.height
                            } else {
                                (event.position.x - bounds.origin.x - drag_pos.x)
                                    / bounds.size.width
                            }
                            .min(1.);

                            let offset = if axis.is_vertical() {
                                point(
                                    thumb_bounds.origin.x,
                                    -percentage * scroll_area_size, // - bounds.origin.y
                                )
                            } else {
                                point(
                                    -percentage * scroll_area_size
                                        // - bounds.origin.x
                                        + thumb_bounds.origin.x,
                                    thumb_bounds.origin.y,
                                )
                            };

                            scroll_handle.set_offset(offset);
                            cx.notify(view_id);
                        }
                    }
                });

                cx.on_mouse_event({
                    let view_id = self.view.entity_id();
                    let state = self.state.clone();

                    move |_event: &MouseUpEvent, phase, cx| {
                        if phase.bubble() {
                            state.set(state.get().unset_drag_pos());
                            cx.notify(view_id);
                        }
                    }
                });
            }
        });
    }
}
