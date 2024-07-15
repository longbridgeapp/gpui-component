use std::{cell::Cell, ops::Range, rc::Rc};

use crate::{
    blue_100, red_200,
    theme::{ActiveTheme, Colorize},
    StyledExt,
};
use gpui::{
    deferred, div, fill, point, prelude::FluentBuilder as _, px, relative, AnyElement, AnyView,
    Axis, Bounds, ContentMask, Element, Hitbox, InteractiveElement as _, IntoElement, LayoutId,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Position, ScrollHandle,
    ScrollWheelEvent, Style, Styled as _, Task,
};

const MIN_THUMB_SIZE: f32 = 80.;
const THUMB_RADIUS: Pixels = Pixels(5.0);
const THUMB_INSET: Pixels = Pixels(0.8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
    Vertical,
    Horizontal,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarState {
    hovered: bool,
    mouse_down: bool,
    visible: bool,
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self {
            hovered: false,
            mouse_down: false,
            visible: true,
        }
    }
}

impl ScrollbarAxis {
    fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
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
                    bounds.size.width,
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

            cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
                let bar_bg = cx.theme().scrollbar;
                let thumb_bg = cx.theme().scrollbar_thumb;
                let state = self.state.clone();
                let thumb_bg = if state.take().hovered {
                    red_200()
                } else {
                    thumb_bg
                };

                // println!("{:?}  thumb_start: {}", axis, thumb_start);
                // println!("{:?}  thumb_end: {}", axis, thumb_end);

                let thumb_bounds = if axis.is_vertical() {
                    Bounds::from_corners(
                        point(
                            bounds.origin.x + THUMB_INSET,
                            bounds.origin.y + thumb_start + THUMB_INSET,
                        ),
                        point(
                            bounds.origin.x + self.width - (THUMB_INSET * 2),
                            bounds.origin.y + thumb_end - (THUMB_INSET * 2),
                        ),
                    )
                } else {
                    Bounds::from_corners(
                        point(
                            bounds.origin.x + thumb_start * THUMB_INSET,
                            bounds.origin.y + THUMB_INSET,
                        ),
                        point(
                            bounds.origin.x + thumb_end - (THUMB_INSET * 2),
                            bounds.origin.y + self.width - (THUMB_INSET * 2),
                        ),
                    )
                };

                cx.paint_quad(fill(bounds, blue_100()));
                cx.paint_quad(fill(thumb_bounds, thumb_bg).corner_radii(THUMB_RADIUS));

                cx.on_mouse_event({
                    let scroll = self.scroll_handle.clone();
                    let state = self.state.clone();
                    let view_id = self.view.entity_id();

                    move |event: &MouseDownEvent, phase, cx| {
                        if phase.bubble() && bounds.contains(&event.position) {
                            if thumb_bounds.contains(&event.position) {
                                println!("-------- thumb_bounds MouseState::Down");
                                state.set(ScrollbarState {
                                    mouse_down: true,
                                    ..state.take()
                                });
                                // cx.notify(view_id);
                            } else {
                                // TODO, click bar to scroll
                            }
                        }
                    }
                });

                cx.on_mouse_event({
                    let scroll_handle = self.scroll_handle.clone();
                    let state = self.state.clone();
                    let view_id = self.view.entity_id();

                    move |event: &MouseMoveEvent, _, cx| {
                        if state.take().mouse_down && event.dragging() {
                            let offset = if axis.is_vertical() {
                                point(scroll_handle.offset().x, event.position.y)
                            } else {
                                point(event.position.x, scroll_handle.offset().y)
                            };

                            println!("-------- dragging scrollbar: {:?}", offset);

                            scroll_handle.set_offset(offset);
                        }
                    }
                });

                cx.on_mouse_event({
                    let view_id = self.view.entity_id();
                    let state = self.state.clone();

                    move |_event: &MouseUpEvent, phase, cx| {
                        if phase.bubble() {
                            state.set(ScrollbarState {
                                mouse_down: false,
                                ..state.take()
                            });
                            // cx.notify(view_id);
                        }
                    }
                });
            });
        }
    }
}
