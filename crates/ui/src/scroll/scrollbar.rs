use std::{cell::Cell, rc::Rc};

use crate::theme::ActiveTheme;
use gpui::{
    fill, point, px, relative, size, Bounds, ContentMask, Edges, Element, EntityId, Hitbox,
    IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, Position,
    ScrollHandle, Style, UniformListScrollHandle,
};

const MIN_THUMB_SIZE: f32 = 80.;
const THUMB_RADIUS: Pixels = Pixels(5.0);
const THUMB_INSET: Pixels = Pixels(2.);

pub trait ScrollHandleOffsetable {
    fn offset(&self) -> Point<Pixels>;
    fn set_offset(&self, offset: Point<Pixels>);
    fn is_uniform_list(&self) -> bool {
        false
    }
}

impl ScrollHandleOffsetable for ScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.set_offset(offset);
    }
}

impl ScrollHandleOffsetable for UniformListScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.0.borrow().base_handle.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.0.borrow_mut().base_handle.set_offset(offset)
    }

    fn is_uniform_list(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarState {
    hovered_axis: Option<ScrollbarAxis>,
    dragged_axis: Option<ScrollbarAxis>,
    drag_pos: Point<Pixels>,
    visible: bool,
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self {
            hovered_axis: None,
            dragged_axis: None,
            drag_pos: point(px(0.), px(0.)),
            visible: false,
        }
    }
}

impl ScrollbarState {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_drag_pos(&self, axis: ScrollbarAxis, pos: Point<Pixels>) -> Self {
        let mut state = *self;
        if axis.is_vertical() {
            state.drag_pos.y = pos.y;
        } else {
            state.drag_pos.x = pos.x;
        }

        state.dragged_axis = Some(axis);
        state
    }

    fn with_unset_drag_pos(&self) -> Self {
        let mut state = *self;
        state.dragged_axis = None;
        state
    }

    fn with_hovered(&self, axis: Option<ScrollbarAxis>) -> Self {
        let mut state = *self;
        state.hovered_axis = axis;
        state
    }

    fn with_visiable(&self, visiable: bool) -> Self {
        let mut state = *self;
        state.visible = visiable;
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
    #[inline]
    fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    #[inline]
    fn is_both(&self) -> bool {
        matches!(self, Self::Both)
    }

    #[inline]
    pub fn has_vertical(&self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }

    #[inline]
    pub fn has_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    #[inline]
    fn all(&self) -> Vec<ScrollbarAxis> {
        match self {
            Self::Vertical => vec![Self::Vertical],
            Self::Horizontal => vec![Self::Horizontal],
            // This should keep vertical first, vertical is the primary axis
            // if vertical not need display, then horizontal will not keep right margin.
            Self::Both => vec![Self::Vertical, Self::Horizontal],
        }
    }
}

/// Scrollbar control for scroll-area or a uniform-list.
pub struct Scrollbar {
    view_id: EntityId,
    axis: ScrollbarAxis,
    /// When is vertical, this is the height of the scrollbar.
    width: Pixels,
    scroll_handle: Rc<Box<dyn ScrollHandleOffsetable>>,
    scroll_size: gpui::Size<Pixels>,
    state: Rc<Cell<ScrollbarState>>,
}

impl Scrollbar {
    fn new(
        view_id: EntityId,
        state: Rc<Cell<ScrollbarState>>,
        axis: ScrollbarAxis,
        scroll_handle: impl ScrollHandleOffsetable + 'static,
        scroll_size: gpui::Size<Pixels>,
    ) -> Self {
        Self {
            view_id,
            state,
            axis,
            scroll_size,
            width: px(11.),
            scroll_handle: Rc::new(Box::new(scroll_handle)),
        }
    }

    /// Create with vertical and horizontal scrollbar.
    pub fn both(
        view_id: EntityId,
        state: Rc<Cell<ScrollbarState>>,
        scroll_handle: impl ScrollHandleOffsetable + 'static,
        scroll_size: gpui::Size<Pixels>,
    ) -> Self {
        Self::new(
            view_id,
            state,
            ScrollbarAxis::Both,
            scroll_handle,
            scroll_size,
        )
    }

    /// Create with horizontal scrollbar.
    pub fn horizontal(
        view_id: EntityId,
        state: Rc<Cell<ScrollbarState>>,
        scroll_handle: impl ScrollHandleOffsetable + 'static,
        scroll_size: gpui::Size<Pixels>,
    ) -> Self {
        Self::new(
            view_id,
            state,
            ScrollbarAxis::Horizontal,
            scroll_handle,
            scroll_size,
        )
    }

    /// Create with vertical scrollbar.
    pub fn vertical(
        view_id: EntityId,
        state: Rc<Cell<ScrollbarState>>,
        scroll_handle: impl ScrollHandleOffsetable + 'static,
        scroll_size: gpui::Size<Pixels>,
    ) -> Self {
        Self::new(
            view_id,
            state,
            ScrollbarAxis::Vertical,
            scroll_handle,
            scroll_size,
        )
    }

    /// Create vertical scrollbar for uniform list.
    pub fn uniform_scroll(
        view_id: EntityId,
        state: Rc<Cell<ScrollbarState>>,
        scroll_handle: UniformListScrollHandle,
        items_count: usize,
    ) -> Self {
        let last_item_height = scroll_handle.0.borrow().last_item_height.unwrap_or(px(10.));
        let max_height = items_count as f32 * last_item_height;
        let scroll_size = size(px(0.), max_height);

        Self::new(
            view_id,
            state,
            ScrollbarAxis::Vertical,
            scroll_handle,
            scroll_size,
        )
    }

    /// Set scrollbar axis.
    pub fn axis(mut self, axis: ScrollbarAxis) -> Self {
        self.axis = axis;
        self
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
        let hitbox_bounds = hitbox.bounds;
        let mut has_both = self.axis.is_both();

        cx.with_content_mask(
            Some(ContentMask {
                bounds: hitbox_bounds,
            }),
            |cx| {
                for axis in self.axis.all().into_iter() {
                    let is_vertical = axis.is_vertical();
                    let (scroll_area_size, container_size, scroll_position) = if is_vertical {
                        (
                            self.scroll_size.height,
                            hitbox_bounds.size.height,
                            self.scroll_handle.offset().y,
                        )
                    } else {
                        (
                            self.scroll_size.width,
                            hitbox_bounds.size.width,
                            self.scroll_handle.offset().x,
                        )
                    };

                    // The horizontal scrollbar is set avoid overlapping with the vertical scrollbar, if the vertical scrollbar is visible.
                    let margin_end = if has_both && !is_vertical {
                        self.width
                    } else {
                        px(0.)
                    };

                    // Hide scrollbar, if the scroll area is smaller than the container.
                    if scroll_area_size <= container_size {
                        has_both = false;
                        continue;
                    }

                    let thumb_length = (container_size / scroll_area_size * container_size)
                        .max(px(MIN_THUMB_SIZE));
                    let thumb_start = -(scroll_position / (scroll_area_size - container_size)
                        * (container_size - margin_end - thumb_length));
                    let thumb_end = (thumb_start + thumb_length).min(container_size - margin_end);

                    let bounds = Bounds {
                        origin: if is_vertical {
                            point(
                                hitbox_bounds.origin.x + hitbox_bounds.size.width - self.width,
                                hitbox_bounds.origin.y,
                            )
                        } else {
                            point(
                                hitbox_bounds.origin.x,
                                hitbox_bounds.origin.y + hitbox_bounds.size.height - self.width,
                            )
                        },
                        size: gpui::Size {
                            width: if is_vertical {
                                self.width
                            } else {
                                hitbox_bounds.size.width
                            },
                            height: if is_vertical {
                                hitbox_bounds.size.height
                            } else {
                                self.width
                            },
                        },
                    };

                    let thumb_bg = cx.theme().scrollbar_thumb;
                    let state = self.state.clone();
                    let (thumb_bg, bar_bg, bar_border, inset) =
                        if state.get().dragged_axis == Some(axis) {
                            (thumb_bg, cx.theme().scrollbar, cx.theme().border, px(1.))
                        } else if state.get().hovered_axis == Some(axis) {
                            (thumb_bg, cx.theme().scrollbar, cx.theme().border, px(1.))
                        } else {
                            (
                                thumb_bg,
                                cx.theme().transparent,
                                gpui::transparent_black(),
                                THUMB_INSET,
                            )
                        };

                    let border_width = px(1.);
                    let thumb_bounds = if is_vertical {
                        Bounds::from_corners(
                            point(
                                bounds.origin.x + inset + border_width,
                                bounds.origin.y + thumb_start,
                            ),
                            point(
                                bounds.origin.x + self.width - inset,
                                bounds.origin.y + thumb_end,
                            ),
                        )
                    } else {
                        Bounds::from_corners(
                            point(
                                bounds.origin.x + thumb_start,
                                bounds.origin.y + inset + border_width,
                            ),
                            point(
                                bounds.origin.x + thumb_end,
                                bounds.origin.y + self.width - inset,
                            ),
                        )
                    };

                    if state.get().visible {
                        cx.paint_quad(fill(bounds, bar_bg));

                        cx.paint_quad(PaintQuad {
                            bounds,
                            corner_radii: (0.).into(),
                            background: gpui::transparent_black(),
                            border_widths: if is_vertical {
                                Edges {
                                    top: px(0.),
                                    right: px(0.),
                                    bottom: px(0.),
                                    left: border_width,
                                }
                            } else {
                                Edges {
                                    top: px(1.),
                                    right: px(0.),
                                    bottom: px(0.),
                                    left: border_width,
                                }
                            },
                            border_color: bar_border,
                        });

                        cx.paint_quad(
                            fill(thumb_bounds, thumb_bg).corner_radii(THUMB_RADIUS - inset),
                        );
                    }

                    cx.on_mouse_event({
                        let state = self.state.clone();
                        let view_id = self.view_id;
                        let scroll_handle = self.scroll_handle.clone();

                        move |event: &MouseDownEvent, phase, cx| {
                            if phase.bubble() && bounds.contains(&event.position) {
                                cx.stop_propagation();

                                if thumb_bounds.contains(&event.position) {
                                    // click on the thumb bar, set the drag position
                                    let pos = event.position - thumb_bounds.origin;

                                    state.set(state.get().with_drag_pos(axis, pos));

                                    cx.notify(view_id);
                                } else {
                                    // click on the scrollbar, jump to the position
                                    // Set the thumb bar center to the click position
                                    let offset = scroll_handle.offset();
                                    let percentage = if is_vertical {
                                        (event.position.y - thumb_length / 2. - bounds.origin.y)
                                            / (bounds.size.height - thumb_length)
                                    } else {
                                        (event.position.x - thumb_length / 2. - bounds.origin.x)
                                            / (bounds.size.width - thumb_length)
                                    }
                                    .min(1.);

                                    if is_vertical {
                                        scroll_handle.set_offset(point(
                                            offset.x,
                                            -scroll_area_size * percentage,
                                        ));
                                    } else {
                                        scroll_handle.set_offset(point(
                                            -scroll_area_size * percentage,
                                            offset.y,
                                        ));
                                    }
                                }
                            }
                        }
                    });

                    cx.on_mouse_event({
                        let scroll_handle = self.scroll_handle.clone();
                        let state = self.state.clone();
                        let view_id = self.view_id;

                        move |event: &MouseMoveEvent, _, cx| {
                            if bounds.contains(&event.position) {
                                if state.get().hovered_axis != Some(axis) {
                                    state.set(state.get().with_hovered(Some(axis)));

                                    cx.notify(view_id);
                                }
                            } else {
                                if state.get().hovered_axis == Some(axis) {
                                    if state.get().hovered_axis.is_some() {
                                        state.set(state.get().with_hovered(None));

                                        cx.notify(view_id);
                                    }
                                }
                            }

                            // If mouse out of the bounds, hide scrollbar
                            if hitbox_bounds.contains(&event.position)
                                || state.get().dragged_axis.is_some()
                            {
                                if !state.get().visible {
                                    state.set(state.get().with_visiable(true));
                                    cx.notify(view_id);
                                }
                            } else {
                                if state.get().visible {
                                    state.set(state.get().with_visiable(false));
                                    cx.notify(view_id);
                                }
                            }

                            // Move thumb position on dragging
                            if state.get().dragged_axis == Some(axis) && event.dragging() {
                                // drag_pos is the position of the mouse down event
                                // We need to keep the thumb bar still at the origin down position
                                let drag_pos = state.get().drag_pos;

                                let percentage = if is_vertical {
                                    (event.position.y - drag_pos.y - bounds.origin.y)
                                        / (bounds.size.height - thumb_length)
                                } else {
                                    (event.position.x - drag_pos.x - bounds.origin.x)
                                        / (bounds.size.width - thumb_length)
                                }
                                .min(1.);

                                let offset = if is_vertical {
                                    point(scroll_handle.offset().x, -scroll_area_size * percentage)
                                } else {
                                    point(-scroll_area_size * percentage, scroll_handle.offset().y)
                                };

                                scroll_handle.set_offset(offset);
                                cx.notify(view_id);
                            }
                        }
                    });

                    cx.on_mouse_event({
                        let view_id = self.view_id;
                        let state = self.state.clone();

                        move |_event: &MouseUpEvent, phase, cx| {
                            if phase.bubble() {
                                state.set(state.get().with_unset_drag_pos());

                                cx.notify(view_id);
                            }
                        }
                    });
                }
            },
        );
    }
}
