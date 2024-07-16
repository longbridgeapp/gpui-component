use std::{cell::Cell, rc::Rc};

use crate::theme::{ActiveTheme, Colorize};
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
    fn with_drag_pos(&self, axis: ScrollbarAxis, pos: Point<Pixels>) -> Self {
        let mut state = *self;
        if axis.is_vertical() {
            state.drag_pos.y = pos.y;
        } else {
            state.drag_pos.x = pos.x;
        }

        state.draged_axis = Some(axis);
        state
    }

    fn with_unset_drag_pos(&self) -> Self {
        let mut state = *self;
        state.draged_axis = None;
        state
    }

    fn with_hovered(&self, axis: Option<ScrollbarAxis>) -> Self {
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
        let is_both = self.axis.is_both();

        // cx.paint_quad(
        //     fill(bounds, gpui::transparent_black())
        //         .border_color(pink_500())
        //         .border_widths(1.0),
        // );

        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            for axis in self.axis.all().into_iter() {
                let is_vertical = axis.is_vertical();
                let (scroll_area_size, container_size, scroll_position) = if is_vertical {
                    (
                        self.scroll_size.height,
                        bounds.size.height,
                        self.scroll_handle.offset().y,
                    )
                } else {
                    (
                        self.scroll_size.width,
                        bounds.size.width,
                        self.scroll_handle.offset().x,
                    )
                };

                // The horizontal scrollbar is set avoid overlapping with the vertical scrollbar, if the vertical scrollbar is visible.
                let margin_end = if !is_vertical && is_both {
                    self.width
                } else {
                    px(0.)
                };

                let thumb_length =
                    (container_size / scroll_area_size * container_size).max(px(MIN_THUMB_SIZE));
                let thumb_start = -(scroll_position / (scroll_area_size - container_size)
                    * (container_size - margin_end - thumb_length));
                let thumb_end = (thumb_start + thumb_length).min(container_size - margin_end);

                let bounds = Bounds {
                    origin: if is_vertical {
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
                        width: if is_vertical {
                            self.width
                        } else {
                            bounds.size.width
                        },
                        height: if is_vertical {
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

                let thumb_bounds = if is_vertical {
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
                            bounds.origin.x + thumb_start + inset,
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
                            let drag_pos = if is_vertical {
                                point(
                                    state.get().drag_pos.x,
                                    event.position.y - thumb_bounds.origin.y,
                                )
                            } else {
                                point(
                                    event.position.x - thumb_bounds.origin.x,
                                    state.get().drag_pos.y,
                                )
                            };

                            state.set(state.get().with_drag_pos(axis, drag_pos));
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
                                state.set(state.get().with_hovered(Some(axis)));
                                cx.notify(view_id);
                            }
                        } else {
                            if state.get().hovered_axis == Some(axis) {
                                if state.get().hovered_axis.is_some() {
                                    state.set(state.get().with_hovered(None));
                                    cx.notify(view_id);

                                    // TODO: Start delay 2s to hide the scrollbar.
                                }
                            }
                        }

                        if event.dragging() && state.get().draged_axis == Some(axis) {
                            let drag_pos = state.get().drag_pos;
                            let percentage = if is_vertical {
                                (event.position.y - bounds.origin.y - drag_pos.y) / container_size
                            } else {
                                (event.position.x - bounds.origin.x - drag_pos.x) / container_size
                            }
                            .min(1.);

                            let offset = if is_vertical {
                                point(scroll_handle.offset().x, -percentage * scroll_area_size)
                            } else {
                                point(-percentage * scroll_area_size, scroll_handle.offset().y)
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
                            state.set(state.get().with_unset_drag_pos());
                            cx.notify(view_id);
                        }
                    }
                });
            }
        });
    }
}
