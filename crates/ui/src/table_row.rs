//! Table row component for render a large number of differently sized columns (Must ensure each column width).
//!
//! Only visible columns are rendered for performance reasons.
//!
//! Inspired by uniform_list to rolate vertically to horizontally.
//!
//! https://github.com/zed-industries/zed/blob/0ae1603610ab6b265bdfbee7b8dbc23c5ab06edc/crates/gpui/src/elements/uniform_list.rs
use std::{cmp, ops::Range, rc::Rc};

use gpui::{
    div, point, px, size, AnyElement, AvailableSpace, Bounds, ContentMask, Div, Element, ElementId,
    Hitbox, InteractiveElement, IntoElement, IsZero as _, Pixels, Render, ScrollHandle,
    SharedString, Size, Stateful, StyleRefinement, Styled, View, ViewContext, WindowContext,
};
use smallvec::SmallVec;

use crate::table::ColGroup;

pub(crate) fn table_row<R, V>(
    view: View<V>,
    row_ix: usize,
    col_groups: Rc<Vec<ColGroup>>,
    scroll_handle: ScrollHandle,
    f: impl 'static + Fn(&mut V, Range<usize>, &mut ViewContext<V>) -> Vec<R>,
) -> TableRow
where
    R: IntoElement,
    V: Render,
{
    let id = ElementId::NamedInteger(SharedString::from("table-row"), row_ix);

    let render_range = move |range, cx: &mut WindowContext| {
        view.update(cx, |this, cx| {
            f(this, range, cx)
                .into_iter()
                .map(|component| component.into_any_element())
                .collect()
        })
    };

    TableRow {
        id: id.clone(),
        base: div().id(id).size_full(),
        scroll_handle,
        cols_count: col_groups.len(),
        col_groups,
        render_cols: Box::new(render_range),
    }
}

pub struct TableRow {
    id: ElementId,
    base: Stateful<Div>,
    scroll_handle: ScrollHandle,
    // scroll_handle: ScrollHandle,
    cols_count: usize,
    col_groups: Rc<Vec<ColGroup>>,
    render_cols:
        Box<dyn for<'a> Fn(Range<usize>, &'a mut WindowContext) -> SmallVec<[AnyElement; 64]>>,
}

impl Styled for TableRow {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

/// Frame state used by the [TableRow].
pub struct TableRowFrameState {
    cols: SmallVec<[AnyElement; 32]>,
    // decorations: SmallVec<[AnyElement; 1]>,
}

impl TableRow {
    #[allow(dead_code)]
    fn measure_col(&self, cx: &mut WindowContext) -> Size<Pixels> {
        if self.cols_count == 0 {
            return Size::default();
        }

        let col_ix = self.cols_count - 1;
        let mut items = (self.render_cols)(col_ix..col_ix + 1, cx);
        let Some(mut item_to_measure) = items.pop() else {
            return Size::default();
        };

        let available_space = size(AvailableSpace::MinContent, AvailableSpace::MinContent);
        item_to_measure.layout_as_root(available_space, cx)
    }
}

impl IntoElement for TableRow {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TableRow {
    type RequestLayoutState = TableRowFrameState;
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
            TableRowFrameState {
                cols: SmallVec::new(),
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

        // This is important to get the width of each column to measure the visible columns.
        //
        // So the col must have a width.
        let col_widths = self
            .col_groups
            .iter()
            .map(|col| col.width.0)
            .collect::<Vec<_>>();

        let content_height = padded_bounds.size.height;
        let content_width = px(col_widths.iter().sum::<f32>());
        let content_size = Size {
            width: content_width,
            height: content_height,
        };

        self.base.interactivity().prepaint(
            global_id,
            bounds,
            content_size,
            cx,
            |style, _, hitbox, cx| {
                let mut scroll_offset = self.scroll_handle.offset();
                // dbg!(&scroll_offset);
                let border = style.border_widths.to_pixels(cx.rem_size());
                let padding = style.padding.to_pixels(bounds.size.into(), cx.rem_size());

                let padded_bounds = Bounds::from_corners(
                    bounds.origin + point(border.left + padding.left, border.top),
                    bounds.lower_right() - point(border.right + padding.right, border.bottom),
                );

                if self.cols_count > 0 {
                    let is_scrolled_horizontally = !scroll_offset.x.is_zero();
                    let min_horizontal_scroll_offset = padded_bounds.size.width - content_width;
                    if is_scrolled_horizontally && scroll_offset.x < min_horizontal_scroll_offset {
                        scroll_offset.x = min_horizontal_scroll_offset;
                    }
                    scroll_offset.y = Pixels::ZERO;

                    // Calculate the first and last visible element indices.
                    let mut cumulative_width = 0.0;
                    let mut first_visible_element_ix = 0;
                    for (i, &width) in col_widths.iter().enumerate() {
                        cumulative_width += width;
                        if cumulative_width > -(scroll_offset.x + padding.left).0 {
                            first_visible_element_ix = i;
                            break;
                        }
                    }

                    cumulative_width = 0.0;
                    let mut last_visible_element_ix = 0;
                    for (i, &width) in col_widths.iter().enumerate() {
                        cumulative_width += width;
                        if cumulative_width > (-scroll_offset.x + padded_bounds.size.width).0 {
                            last_visible_element_ix = i + 1;
                            break;
                        }
                    }
                    if last_visible_element_ix == 0 {
                        last_visible_element_ix = self.cols_count;
                    } else {
                        last_visible_element_ix += 1;
                    }
                    let visible_range = first_visible_element_ix
                        ..cmp::min(last_visible_element_ix, self.cols_count);

                    let items = (self.render_cols)(visible_range.clone(), cx);

                    let content_mask = ContentMask { bounds };
                    cx.with_content_mask(Some(content_mask), |cx| {
                        for (mut item, ix) in items.into_iter().zip(visible_range.clone()) {
                            let item_x = px(col_widths.iter().take(ix).sum::<f32>());

                            let item_origin = padded_bounds.origin
                                + point(item_x + scroll_offset.x + padding.left, padding.top);
                            // println!("{}, {}", item_origin.x, item_origin.y);
                            let available_height = padded_bounds.size.height;
                            let col_width = col_widths[ix];
                            let available_space = size(
                                AvailableSpace::Definite(px(col_width)),
                                AvailableSpace::Definite(available_height),
                            );
                            item.layout_as_root(available_space, cx);
                            item.prepaint_at(item_origin, cx);
                            frame_state.cols.push(item);
                        }

                        // let bounds = Bounds::new(
                        //     padded_bounds.origin
                        //         + point(scroll_offset.x + padding.left, scroll_offset.y),
                        //     padded_bounds.size,
                        // );
                        // for decoration in &self.decorations {
                        //     let mut decoration = decoration.as_ref().compute(
                        //         visible_range.clone(),
                        //         bounds,
                        //         item_height,
                        //         self.item_count,
                        //         cx,
                        //     );
                        //     let available_space = size(
                        //         AvailableSpace::Definite(bounds.size.width),
                        //         AvailableSpace::Definite(bounds.size.height),
                        //     );
                        //     decoration.layout_as_root(available_space, cx);
                        //     decoration.prepaint_at(bounds.origin, cx);
                        //     frame_state.decorations.push(decoration);
                        // }
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
                for col in &mut request_layout.cols {
                    col.paint(cx);
                }
            })
    }
}
