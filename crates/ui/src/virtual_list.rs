//! Vistual List for render a large number of differently sized rows/columns.
//!
//! NOTE: This must ensure each column width or row height.
//!
//! Only visible range are rendered for performance reasons.
//!
//! Inspired by `gpui::uniform_list`.
//! https://github.com/zed-industries/zed/blob/0ae1603610ab6b265bdfbee7b8dbc23c5ab06edc/crates/gpui/src/elements/uniform_list.rs
use std::{cmp, ops::Range, rc::Rc};

use gpui::{
    div, point, px, size, AnyElement, AvailableSpace, Axis, Bounds, ContentMask, Div, Element,
    ElementId, Hitbox, InteractiveElement, IntoElement, IsZero as _, Pixels, Render, ScrollHandle,
    Size, Stateful, StyleRefinement, Styled, View, ViewContext, WindowContext,
};
use smallvec::SmallVec;

/// Create a virtual list in Vertical direction.
///
/// This is like `uniform_list` in GPUI, but support two axis.
///
/// The `item_sizes` is the size of each column.
pub fn vertical_virtual_list<R, V>(
    id: impl Into<ElementId>,
    view: View<V>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: ScrollHandle,
    f: impl 'static + Fn(&mut V, Range<usize>, &mut ViewContext<V>) -> Vec<R>,
) -> VirtualItem
where
    R: IntoElement,
    V: Render,
{
    virtual_list(id, view, Axis::Vertical, item_sizes, scroll_handle, f)
}

/// Create a virtual list in Horizontal direction.
pub fn horizontal_virtual_list<R, V>(
    id: impl Into<ElementId>,
    view: View<V>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: ScrollHandle,
    f: impl 'static + Fn(&mut V, Range<usize>, &mut ViewContext<V>) -> Vec<R>,
) -> VirtualItem
where
    R: IntoElement,
    V: Render,
{
    virtual_list(id, view, Axis::Horizontal, item_sizes, scroll_handle, f)
}

fn virtual_list<R, V>(
    id: impl Into<ElementId>,
    view: View<V>,
    axis: Axis,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: ScrollHandle,
    f: impl 'static + Fn(&mut V, Range<usize>, &mut ViewContext<V>) -> Vec<R>,
) -> VirtualItem
where
    R: IntoElement,
    V: Render,
{
    let id: ElementId = id.into();
    let render_range = move |range, cx: &mut WindowContext| {
        view.update(cx, |this, cx| {
            f(this, range, cx)
                .into_iter()
                .map(|component| component.into_any_element())
                .collect()
        })
    };

    VirtualItem {
        id: id.clone(),
        axis,
        base: div().id(id).size_full(),
        scroll_handle,
        items_count: item_sizes.len(),
        item_sizes,
        render_items: Box::new(render_range),
    }
}

/// VirtualItem component for rendering a large number of differently sized columns.
pub struct VirtualItem {
    id: ElementId,
    axis: Axis,
    base: Stateful<Div>,
    scroll_handle: ScrollHandle,
    // scroll_handle: ScrollHandle,
    items_count: usize,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    render_items:
        Box<dyn for<'a> Fn(Range<usize>, &'a mut WindowContext) -> SmallVec<[AnyElement; 64]>>,
}

impl Styled for VirtualItem {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

/// Frame state used by the [TableRow].
pub struct VirtualListFrameState {
    items: SmallVec<[AnyElement; 32]>,
    // decorations: SmallVec<[AnyElement; 1]>,
}

impl VirtualItem {
    #[allow(dead_code)]
    fn measure_col(&self, cx: &mut WindowContext) -> Size<Pixels> {
        if self.items_count == 0 {
            return Size::default();
        }

        let ix = self.items_count - 1;
        let mut items = (self.render_items)(ix..ix + 1, cx);
        let Some(mut item_to_measure) = items.pop() else {
            return Size::default();
        };

        let available_space = size(AvailableSpace::MinContent, AvailableSpace::MinContent);
        item_to_measure.layout_as_root(available_space, cx)
    }
}

impl IntoElement for VirtualItem {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for VirtualItem {
    type RequestLayoutState = VirtualListFrameState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<gpui::ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let (layout_id, _) = self.base.request_layout(global_id, cx);

        (
            layout_id,
            VirtualListFrameState {
                items: SmallVec::new(),
            },
        )
    }

    fn prepaint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        frame_state: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        let style = self.base.interactivity().compute_style(global_id, None, cx);
        let border = style.border_widths.to_pixels(cx.rem_size());
        let padding = style.padding.to_pixels(bounds.size.into(), cx.rem_size());

        let padded_bounds = Bounds::from_corners(
            bounds.origin + point(border.left + padding.left, border.top + padding.top),
            bounds.lower_right()
                - point(border.right + padding.right, border.bottom + padding.bottom),
        );

        let item_sizes = match self.axis {
            Axis::Horizontal => self
                .item_sizes
                .iter()
                .map(|size| size.width.0)
                .collect::<Vec<_>>(),
            Axis::Vertical => self
                .item_sizes
                .iter()
                .map(|size| size.height.0)
                .collect::<Vec<_>>(),
        };

        let content_size = match self.axis {
            Axis::Horizontal => Size {
                width: px(item_sizes.iter().sum::<f32>()),
                height: padded_bounds.size.height,
            },
            Axis::Vertical => Size {
                width: padded_bounds.size.width,
                height: px(item_sizes.iter().sum::<f32>()),
            },
        };

        self.base.interactivity().prepaint(
            global_id,
            bounds,
            content_size,
            cx,
            |style, _, hitbox, cx| {
                let mut scroll_offset = self.scroll_handle.offset();
                let border = style.border_widths.to_pixels(cx.rem_size());
                let padding = style.padding.to_pixels(bounds.size.into(), cx.rem_size());

                let padded_bounds = Bounds::from_corners(
                    bounds.origin + point(border.left + padding.left, border.top),
                    bounds.lower_right() - point(border.right + padding.right, border.bottom),
                );

                if self.items_count > 0 {
                    let is_scrolled = match self.axis {
                        Axis::Horizontal => !scroll_offset.x.is_zero(),
                        Axis::Vertical => !scroll_offset.y.is_zero(),
                    };

                    let min_scroll_offset = match self.axis {
                        Axis::Horizontal => padded_bounds.size.width - content_size.width,
                        Axis::Vertical => padded_bounds.size.height - content_size.height,
                    };

                    if is_scrolled {
                        match self.axis {
                            Axis::Horizontal if scroll_offset.x < min_scroll_offset => {
                                scroll_offset.x = min_scroll_offset;
                            }
                            Axis::Vertical if scroll_offset.y < min_scroll_offset => {
                                scroll_offset.y = min_scroll_offset;
                            }
                            _ => {}
                        }
                    }

                    let (first_visible_element_ix, last_visible_element_ix) = match self.axis {
                        Axis::Horizontal => {
                            scroll_offset.y = Pixels::ZERO;
                            let mut cumulative_size = 0.0;
                            let mut first_visible_element_ix = 0;
                            for (i, &size) in item_sizes.iter().enumerate() {
                                cumulative_size += size;
                                if cumulative_size > -(scroll_offset.x + padding.left).0 {
                                    first_visible_element_ix = i;
                                    break;
                                }
                            }

                            cumulative_size = 0.0;
                            let mut last_visible_element_ix = 0;
                            for (i, &size) in item_sizes.iter().enumerate() {
                                cumulative_size += size;
                                if cumulative_size > (-scroll_offset.x + padded_bounds.size.width).0
                                {
                                    last_visible_element_ix = i + 1;
                                    break;
                                }
                            }
                            if last_visible_element_ix == 0 {
                                last_visible_element_ix = self.items_count;
                            } else {
                                last_visible_element_ix += 1;
                            }
                            (first_visible_element_ix, last_visible_element_ix)
                        }
                        Axis::Vertical => {
                            scroll_offset.x = Pixels::ZERO;
                            let mut cumulative_size = 0.0;
                            let mut first_visible_element_ix = 0;
                            for (i, &size) in item_sizes.iter().enumerate() {
                                cumulative_size += size;
                                if cumulative_size > -(scroll_offset.y + padding.top).0 {
                                    first_visible_element_ix = i;
                                    break;
                                }
                            }

                            cumulative_size = 0.0;
                            let mut last_visible_element_ix = 0;
                            for (i, &size) in item_sizes.iter().enumerate() {
                                cumulative_size += size;
                                if cumulative_size
                                    > (-scroll_offset.y + padded_bounds.size.height).0
                                {
                                    last_visible_element_ix = i + 1;
                                    break;
                                }
                            }
                            if last_visible_element_ix == 0 {
                                last_visible_element_ix = self.items_count;
                            } else {
                                last_visible_element_ix += 1;
                            }
                            (first_visible_element_ix, last_visible_element_ix)
                        }
                    };

                    let visible_range = first_visible_element_ix
                        ..cmp::min(last_visible_element_ix, self.items_count);

                    let items = (self.render_items)(visible_range.clone(), cx);

                    let content_mask = ContentMask { bounds };
                    cx.with_content_mask(Some(content_mask), |cx| {
                        for (mut item, ix) in items.into_iter().zip(visible_range.clone()) {
                            let item_origin = match self.axis {
                                Axis::Horizontal => {
                                    let item_x = px(item_sizes.iter().take(ix).sum::<f32>());
                                    padded_bounds.origin
                                        + point(
                                            item_x + scroll_offset.x + padding.left,
                                            padding.top,
                                        )
                                }
                                Axis::Vertical => {
                                    let item_y = px(item_sizes.iter().take(ix).sum::<f32>());
                                    padded_bounds.origin
                                        + point(
                                            padding.left,
                                            item_y + scroll_offset.y + padding.top,
                                        )
                                }
                            };

                            let available_space = match self.axis {
                                Axis::Horizontal => size(
                                    AvailableSpace::Definite(px(item_sizes[ix])),
                                    AvailableSpace::Definite(padded_bounds.size.height),
                                ),
                                Axis::Vertical => size(
                                    AvailableSpace::Definite(padded_bounds.size.width),
                                    AvailableSpace::Definite(px(item_sizes[ix])),
                                ),
                            };

                            item.layout_as_root(available_space, cx);
                            item.prepaint_at(item_origin, cx);
                            frame_state.items.push(item);
                        }
                    });
                }

                hitbox
            },
        )
    }

    fn paint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        self.base
            .interactivity()
            .paint(global_id, bounds, hitbox.as_ref(), cx, |_, cx| {
                for col in &mut request_layout.items {
                    col.paint(cx);
                }
            })
    }
}
