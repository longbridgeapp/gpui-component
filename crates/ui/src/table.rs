use std::{cell::Cell, rc::Rc};

use crate::{
    h_flex,
    scroll::{ScrollableAxis, ScrollableMask, Scrollbar, ScrollbarState},
    theme::ActiveTheme,
    v_flex,
};
use gpui::{
    actions, canvas, div, prelude::FluentBuilder as _, px, uniform_list, AppContext, Bounds, Div,
    DragMoveEvent, EntityId, EventEmitter, FocusHandle, FocusableView, InteractiveElement as _,
    IntoElement, KeyBinding, MouseButton, ParentElement as _, Pixels, Render, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled, UniformListScrollHandle, ViewContext,
    VisualContext as _, WindowContext,
};

actions!(
    table,
    [
        Cancel,
        SelectPrev,
        SelectNext,
        SelectPrevColumn,
        SelectNextColumn
    ]
);

pub fn init(cx: &mut AppContext) {
    let context = Some("Table");
    cx.bind_keys([
        KeyBinding::new("escape", Cancel, context),
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("left", SelectPrevColumn, context),
        KeyBinding::new("right", SelectNextColumn, context),
    ]);
}

struct ColGroup {
    width: Option<Pixels>,
    bounds: Bounds<Pixels>,
}

#[derive(Clone, Render)]
pub struct DragCol(pub (EntityId, usize));

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SelectionState {
    Column,
    Row,
}

#[derive(Clone)]
pub enum TableEvent {
    SelectRow(usize),
    SelectCol(usize),
    ColWidthsChanged(Vec<Option<Pixels>>),
}

pub struct Table<D: TableDelegate> {
    focus_handle: FocusHandle,
    delegate: D,
    horizontal_scroll_handle: ScrollHandle,
    vertical_scroll_handle: UniformListScrollHandle,
    col_groups: Vec<ColGroup>,

    scrollbar_state: Rc<Cell<ScrollbarState>>,

    selection_state: SelectionState,
    selected_row: Option<usize>,
    selected_col: Option<usize>,

    /// The column index that is being resized.
    resizing_col: Option<usize>,
}

#[allow(unused)]
pub trait TableDelegate: Sized + 'static {
    /// Return the number of columns in the table.
    fn cols_count(&self) -> usize;
    /// Return the number of rows in the table.
    fn rows_count(&self) -> usize;

    /// Returns the name of the column at the given index.
    fn column_name(&self, col_ix: usize) -> SharedString;

    /// Returns whether the column at the given index can be resized. Default: true
    fn can_resize_col(&self, col_ix: usize) -> bool {
        true
    }

    /// Returns the width of the column at the given index.
    /// Return None, use auto width.
    ///
    /// This is only called when the table initializes.
    fn col_width(&self, col_ix: usize) -> Option<Pixels>;

    /// When the column has resized, this method is called.
    fn on_col_widths_changed(&mut self, col_widths: Vec<Option<Pixels>>) {}

    /// Render the header cell at the given column index, default to the column name.
    fn render_th(&self, col_ix: usize) -> impl IntoElement {
        div().size_full().child(self.column_name(col_ix))
    }

    /// Render cell at the given row and column.
    fn render_td(&self, row_ix: usize, col_ix: usize) -> impl IntoElement;

    /// Return true to enable loop selection on the table.
    ///
    /// When the prev/next selection is out of the table bounds, the selection will loop to the other side.
    ///
    /// Default: true
    fn can_loop_select(&self) -> bool {
        true
    }
}

impl<D> Table<D>
where
    D: TableDelegate,
{
    pub fn new(delegate: D, cx: &mut ViewContext<Self>) -> Self {
        let mut this = Self {
            focus_handle: cx.focus_handle(),
            delegate,
            col_groups: Vec::new(),
            horizontal_scroll_handle: ScrollHandle::new(),
            vertical_scroll_handle: UniformListScrollHandle::new(),
            scrollbar_state: Rc::new(Cell::new(ScrollbarState::new())),
            selection_state: SelectionState::Row,
            selected_row: None,
            selected_col: None,
            resizing_col: None,
        };

        this.prepare_col_groups(cx);
        this
    }

    pub fn delegate(&self) -> &D {
        &self.delegate
    }

    pub fn delegate_mut(&mut self) -> &mut D {
        &mut self.delegate
    }

    fn prepare_col_groups(&mut self, cx: &mut ViewContext<Self>) {
        self.col_groups = (0..self.delegate.cols_count())
            .map(|col_ix| ColGroup {
                width: self.delegate.col_width(col_ix),
                bounds: Bounds::default(),
            })
            .collect();
        cx.notify();
    }

    fn set_selected_row(&mut self, row_ix: usize, cx: &mut ViewContext<Self>) {
        self.selection_state = SelectionState::Row;
        self.selected_row = Some(row_ix);
        if let Some(row_ix) = self.selected_row {
            self.vertical_scroll_handle.scroll_to_item(row_ix);
        }
        cx.emit(TableEvent::SelectRow(row_ix));
        cx.notify();
    }

    fn set_selected_col(&mut self, col_ix: usize, cx: &mut ViewContext<Self>) {
        self.selection_state = SelectionState::Column;
        self.selected_col = Some(col_ix);
        if let Some(col_ix) = self.selected_col {
            self.horizontal_scroll_handle.scroll_to_item(col_ix);
        }
        cx.emit(TableEvent::SelectCol(col_ix));
        cx.notify();
    }

    fn on_row_click(&mut self, row_ix: usize, cx: &mut ViewContext<Self>) {
        self.set_selected_row(row_ix, cx)
    }

    fn on_col_head_click(&mut self, col_ix: usize, cx: &mut ViewContext<Self>) {
        self.set_selected_col(col_ix, cx)
    }

    fn action_cancel(&mut self, _: &Cancel, cx: &mut ViewContext<Self>) {
        self.selection_state = SelectionState::Row;
        self.selected_row = None;
        self.selected_col = None;
        cx.notify();
    }

    fn action_select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        let mut selected_row = self.selected_row.unwrap_or(0);
        let rows_count = self.delegate.rows_count();
        if selected_row > 0 {
            selected_row = selected_row - 1;
        } else {
            if self.delegate.can_loop_select() {
                selected_row = rows_count - 1;
            }
        }

        self.set_selected_row(selected_row, cx);
    }

    fn action_select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        let mut selected_row = self.selected_row.unwrap_or(0);
        if selected_row < self.delegate.rows_count() - 1 {
            selected_row += 1;
        } else {
            if self.delegate.can_loop_select() {
                selected_row = 0;
            }
        }

        self.set_selected_row(selected_row, cx);
    }

    fn action_select_prev_col(&mut self, _: &SelectPrevColumn, cx: &mut ViewContext<Self>) {
        let mut selected_col = self.selected_col.unwrap_or(0);
        let cols_count = self.delegate.cols_count();
        if selected_col > 0 {
            selected_col -= 1;
        } else {
            if self.delegate.can_loop_select() {
                selected_col = cols_count - 1;
            }
        }
        self.set_selected_col(selected_col, cx);
    }

    fn action_select_next_col(&mut self, _: &SelectNextColumn, cx: &mut ViewContext<Self>) {
        let mut selected_col = self.selected_col.unwrap_or(0);
        if selected_col < self.delegate.cols_count() - 1 {
            selected_col += 1;
        } else {
            if self.delegate.can_loop_select() {
                selected_col = 0;
            }
        }

        self.set_selected_col(selected_col, cx);
    }

    fn render_cell(&self, col_ix: usize, _cx: &mut ViewContext<Self>) -> Div {
        let col_width = self.col_groups[col_ix].width;

        div()
            .when_some(col_width, |this, width| this.w(width))
            .overflow_hidden()
            .whitespace_nowrap()
            .py_1()
            .px_2()
    }

    /// Show Column selection style, when the column is selected and the selection state is Column.
    fn col_wrap(&self, col_ix: usize, cx: &mut ViewContext<Self>) -> Div {
        if self.selected_col == Some(col_ix) && self.selection_state == SelectionState::Column {
            h_flex().bg(cx.theme().table_active)
        } else {
            h_flex()
        }
    }

    fn render_scrollbar(&self, cx: &mut ViewContext<Self>) -> Option<impl IntoElement> {
        let view = cx.view().clone();
        let state = self.scrollbar_state.clone();

        Some(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .child(Scrollbar::uniform_scroll(
                    view,
                    state,
                    self.vertical_scroll_handle.clone(),
                    self.delegate.rows_count(),
                )),
        )
    }

    fn render_resize_handle(&self, ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        const HANDLE_SIZE: Pixels = px(3.);
        if ix > self.col_groups.len() - 1 {
            return div().into_any_element();
        }

        if !self.delegate.can_resize_col(ix) {
            return div().into_any_element();
        }

        let group_id: SharedString = format!("resizable-handle-{}", ix).into();
        let is_resizing = self.resizing_col == Some(ix);

        h_flex()
            .id(("resizable-handle", ix))
            .group(group_id.clone())
            .occlude()
            .cursor_col_resize()
            .h_full()
            .w(HANDLE_SIZE)
            .ml(-(HANDLE_SIZE))
            .justify_end()
            .items_center()
            .child(
                div()
                    .h_full()
                    .h_6()
                    .justify_center()
                    .bg(cx.theme().border)
                    .when(is_resizing, |this| this.bg(cx.theme().drag_border))
                    .group_hover(group_id, |this| this.bg(cx.theme().drag_border))
                    .w(px(1.)),
            )
            .hover(|this| this.bg(cx.theme().drag_border))
            .when(is_resizing, |this| this.bg(cx.theme().drag_border))
            .on_drag_move(cx.listener(
                move |view, e: &DragMoveEvent<DragCol>, cx| match e.drag(cx) {
                    DragCol((entity_id, ix)) => {
                        if cx.entity_id() != *entity_id {
                            return;
                        }

                        // sync col widths into real widths
                        for (_, col_group) in view.col_groups.iter_mut().enumerate() {
                            col_group.width = Some(col_group.bounds.size.width);
                        }

                        let ix = *ix;
                        view.resizing_col = Some(ix);

                        let col_group = view.col_groups.get(ix).expect("BUG: invalid col index");
                        view.resize_cols(
                            ix,
                            e.event.position.x - HANDLE_SIZE - col_group.bounds.left(),
                            cx,
                        );
                    }
                },
            ))
            .on_drag(DragCol((cx.entity_id(), ix)), |drag, cx| {
                cx.new_view(|_| drag.clone())
            })
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|view, _, cx| {
                    if view.resizing_col.is_none() {
                        return;
                    }

                    view.resizing_col = None;

                    let new_widths = view.col_groups.iter().map(|g| g.width).collect();
                    cx.emit(TableEvent::ColWidthsChanged(new_widths));
                    cx.notify();
                }),
            )
            .into_any_element()
    }

    /// The `ix`` is the index of the col to resize,
    /// and the `size` is the new size for the col.
    fn resize_cols(&mut self, ix: usize, size: Pixels, cx: &mut ViewContext<Self>) {
        const MIN_WIDTH: Pixels = px(10.0);
        // Only resize the left cols.
        if ix == self.col_groups.len() - 1 {
            return;
        }
        if !self.delegate.can_resize_col(ix) {
            return;
        }
        let size = size.floor();

        let old_width = self.col_groups[ix].width.unwrap_or_default();
        let new_width = size;
        if new_width < MIN_WIDTH {
            return;
        }
        let changed_width = new_width - old_width;
        // If change size is less than 1px, do nothing.
        if changed_width > px(-1.0) && changed_width < px(1.0) {
            return;
        }
        self.col_groups[ix].width = Some(new_width);

        // Resize next col, table not need to resize the right cols.
        // let next_width = self.col_groups[ix + 1].width.unwrap_or_default();
        // let next_width = (next_width - changed_width).max(MIN_WIDTH);
        // self.col_groups[ix + 1].width = Some(next_width);

        cx.notify();
    }

    /// Render the column header.
    /// The children must be one by one items.
    /// Becuase the horizontal scroll handle will use the child_item_bounds to
    /// calculate the item position for itself's `scroll_to_item` method.
    fn render_th(&self, col_ix: usize, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.col_wrap(col_ix, cx)
            .child(
                self.render_cell(col_ix, cx)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, cx| {
                            this.on_col_head_click(col_ix, cx);
                        }),
                    )
                    .child(self.delegate.render_th(col_ix)),
            )
            // resize handle
            .child(self.render_resize_handle(col_ix, cx))
            // to save the bounds of this col.
            .child({
                let view = cx.view().clone();
                canvas(
                    move |bounds, cx| view.update(cx, |r, _| r.col_groups[col_ix].bounds = bounds),
                    |_, _, _| {},
                )
                .absolute()
                .size_full()
            })
    }
}

impl<D> FocusableView for Table<D>
where
    D: TableDelegate,
{
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<D> EventEmitter<TableEvent> for Table<D> where D: TableDelegate {}

impl<D> Render for Table<D>
where
    D: TableDelegate,
{
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let view = cx.view().clone();
        let vertical_scroll_handle = self.vertical_scroll_handle.clone();
        let horizontal_scroll_handle = self.horizontal_scroll_handle.clone();
        let cols_count: usize = self.delegate.cols_count();
        let rows_count = self.delegate.rows_count();

        fn tr(_: &mut WindowContext) -> Div {
            h_flex()
        }

        let inner_table = v_flex()
            .key_context("Table")
            .id("table")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::action_cancel))
            .on_action(cx.listener(Self::action_select_next))
            .on_action(cx.listener(Self::action_select_prev))
            .on_action(cx.listener(Self::action_select_next_col))
            .on_action(cx.listener(Self::action_select_prev_col))
            .size_full()
            .overflow_hidden()
            .child(
                v_flex()
                    .flex_grow()
                    .h_10()
                    .w_full()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        uniform_list(view.clone(), "table-uniform-list-head", 1, {
                            let horizontal_scroll_handle = horizontal_scroll_handle.clone();
                            move |table, _, cx| {
                                // Columns
                                tr(cx)
                                    .id("table-head")
                                    .w_full()
                                    .h_10()
                                    .overflow_scroll()
                                    .track_scroll(&horizontal_scroll_handle)
                                    .bg(cx.theme().table_head)
                                    .children(
                                        table
                                            .col_groups
                                            .iter()
                                            .enumerate()
                                            .map(|(col_ix, _)| table.render_th(col_ix, cx)),
                                    )
                                    .map(|this| vec![this])
                            }
                        })
                        .size_full(),
                    ),
            )
            .child(
                h_flex().id("table-body").flex_grow().size_full().child(
                    uniform_list(view, "table-uniform-list", rows_count, {
                        let horizontal_scroll_handle = horizontal_scroll_handle.clone();
                        move |table, visible_range, cx| {
                            visible_range
                                .map(|row_ix| {
                                    tr(cx)
                                        .id(("table-row", row_ix))
                                        .w_full()
                                        .children((0..cols_count).map(|col_ix| {
                                            table
                                                .col_wrap(col_ix, cx) // Make the row scroll sync with the horizontal_scroll_handle to support horizontal scrolling.
                                                .left(horizontal_scroll_handle.offset().x)
                                                .child(
                                                    table
                                                        .render_cell(col_ix, cx)
                                                        .flex_shrink_0()
                                                        .child(
                                                            table
                                                                .delegate
                                                                .render_td(row_ix, col_ix),
                                                        ),
                                                )
                                        }))
                                        .when(row_ix > 0, |this| this.border_t_1())
                                        .when(row_ix % 2 == 0, |this| {
                                            this.bg(cx.theme().table_even)
                                        })
                                        .hover(|this| {
                                            if table.selected_row.is_some() {
                                                this
                                            } else {
                                                this.bg(cx.theme().table_hover)
                                            }
                                        })
                                        // Row selected style
                                        .when_some(table.selected_row, |this, selected_row| {
                                            this.when(
                                                row_ix == selected_row
                                                    && table.selection_state == SelectionState::Row,
                                                |this| this.bg(cx.theme().table_active),
                                            )
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, cx| {
                                                this.on_row_click(row_ix, cx);
                                            }),
                                        )
                                })
                                .collect::<Vec<_>>()
                        }
                    })
                    .flex_grow()
                    .size_full()
                    .with_sizing_behavior(gpui::ListSizingBehavior::Auto)
                    .track_scroll(vertical_scroll_handle)
                    .into_any_element(),
                ),
            );

        div()
            .size_full()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().table)
            .child(inner_table)
            .children(self.render_scrollbar(cx))
            .child(ScrollableMask::new(
                cx.view().clone(),
                ScrollableAxis::Horizontal,
                &horizontal_scroll_handle,
            ))
    }
}
