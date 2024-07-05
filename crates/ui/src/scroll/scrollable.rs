use gpui::{
    px, relative, AnyView, Bounds, ContentMask, Corners, Edges, Element, Hitbox, Hsla, IntoElement,
    IsZero as _, PaintQuad, ScrollHandle, ScrollWheelEvent, Style, WindowContext,
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxis {
    Horizontal,
    Vertical,
}

/// Make a scrollable element to cover the parent view with the mouse wheel event listening.
///
/// When the mouse wheel is scrolled, will move the `scroll_handle` to make parent view scrolling with the `axis` direction.
/// This is only can handle once axis scrolling.
pub struct ScrollableHandleElement {
    view: AnyView,
    axis: ScrollAxis,
    scroll_handle: ScrollHandle,
    debug: Option<Hsla>,
}

impl ScrollableHandleElement {
    pub fn new(view: impl Into<AnyView>, axis: ScrollAxis, scroll_handle: &ScrollHandle) -> Self {
        Self {
            view: view.into(),
            debug: None,
            axis,
            scroll_handle: scroll_handle.clone(),
        }
    }

    /// Enable the debug mode to show the scrollable handle bounds.
    #[allow(dead_code)]
    pub fn debug_yellow(mut self) -> Self {
        self.debug = Some(gpui::yellow());
        self
    }
}

impl IntoElement for ScrollableHandleElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ScrollableHandleElement {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        // Set the layout style relative to the table view to get same size.
        style.position = gpui::Position::Absolute;
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();

        (cx.request_layout(style, None), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        // Move y to bounds height to cover the table.
        let cover_bounds = Bounds {
            origin: gpui::Point {
                x: bounds.origin.x,
                y: bounds.origin.y - bounds.size.height,
            },
            size: bounds.size,
        };

        cx.insert_hitbox(cover_bounds, false)
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let line_height = cx.line_height();
        let bounds = hitbox.bounds;

        cx.with_content_mask(Some(ContentMask { bounds }), |cx| {
            if let Some(color) = self.debug {
                cx.paint_quad(PaintQuad {
                    bounds,
                    border_widths: Edges::all(px(1.0)),
                    border_color: color,
                    background: gpui::transparent_white(),
                    corner_radii: Corners::all(px(0.)),
                });
            }

            cx.on_mouse_event({
                let mouse_position = cx.mouse_position();
                let scroll_handle = self.scroll_handle.clone();
                let view_id = self.view.entity_id();
                let is_horizontal = self.axis == ScrollAxis::Horizontal;

                move |event: &ScrollWheelEvent, _, cx| {
                    if bounds.contains(&mouse_position) {
                        let delta = event.delta.pixel_delta(line_height);

                        if is_horizontal && !delta.x.is_zero() {
                            // When is horizontal scroll, move the horizontal scroll handle to make scrolling.
                            let mut offset = scroll_handle.offset();
                            offset.x += delta.x;
                            scroll_handle.set_offset(offset);
                            cx.notify(view_id);
                            cx.stop_propagation();
                        }

                        if !is_horizontal && !delta.y.is_zero() {
                            // When is vertical scroll, move the vertical scroll handle to make scrolling.
                            let mut offset = scroll_handle.offset();
                            offset.y += delta.y;
                            scroll_handle.set_offset(offset);
                            cx.notify(view_id);
                            cx.stop_propagation();
                        }
                    }
                }
            });
        });
    }
}
