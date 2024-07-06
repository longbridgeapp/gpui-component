use std::{cell::Cell, rc::Rc, time::Duration};

use crate::{
    h_flex,
    scroll::{ScrollAxis, ScrollableMask},
    scrollbar::Scrollbar,
    theme::{ActiveTheme, Colorize},
    v_flex,
};
use gpui::{
    actions, deferred, div, prelude::FluentBuilder as _, px, uniform_list, AppContext, Div,
    FocusHandle, FocusableView, InteractiveElement as _, IntoElement, KeyBinding, MouseButton,
    ParentElement as _, Render, ScrollHandle, SharedString, StatefulInteractiveElement as _,
    Styled, Task, UniformListScrollHandle, ViewContext, WindowContext,
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
    width: Option<f32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SelectionState {
    Column,
    Row,
}

pub struct Table<D: TableDelegate> {
    focus_handle: FocusHandle,
    delegate: D,
    horizontal_scroll_handle: ScrollHandle,
    vertical_scroll_handle: UniformListScrollHandle,
    col_groups: Vec<ColGroup>,
    show_scrollbar: bool,
    hide_scrollbar_task: Option<Task<()>>,
    scrollbar_drag_state: Rc<Cell<Option<f32>>>,

    selection_state: SelectionState,
    selected_row: Option<usize>,
    selected_col: Option<usize>,
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
    fn col_width(&self, col_ix: usize) -> Option<f32>;

    /// Set the width of the column at the given index.
    fn on_col_width_changed(&mut self, col_ix: usize, width: Option<f32>) {}

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
            show_scrollbar: false,
            hide_scrollbar_task: None,
            scrollbar_drag_state: Rc::new(Cell::new(None)),
            selection_state: SelectionState::Row,
            selected_row: None,
            selected_col: None,
        };

        this.update_col_groups(cx);
        this
    }

    pub fn delegate(&self) -> &D {
        &self.delegate
    }

    pub fn delegate_mut(&mut self) -> &mut D {
        &mut self.delegate
    }

    fn update_col_groups(&mut self, cx: &mut ViewContext<Self>) {
        self.col_groups = (0..self.delegate.cols_count())
            .map(|col_ix| ColGroup {
                width: self.delegate.col_width(col_ix),
            })
            .collect();
        cx.notify();
    }

    fn render_scrollbar(&self, cx: &mut ViewContext<Self>) -> Option<impl IntoElement> {
        Scrollbar::new(
            cx.view().clone().into(),
            self.vertical_scroll_handle.clone(),
            self.scrollbar_drag_state.clone(),
            self.delegate.rows_count(),
            true,
        )
        .map(|bar| {
            deferred(
                div()
                    .occlude()
                    .absolute()
                    .h_full()
                    .left_auto()
                    .top_0()
                    .right_0()
                    .w(px(bar.width()))
                    .bottom_0()
                    .child(bar),
            )
            .with_priority(1)
        })
    }

    fn hide_scrollbar(&mut self, cx: &mut ViewContext<Self>) {
        self.show_scrollbar = false;
        self.hide_scrollbar_task = Some(cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            this.update(&mut cx, |this, cx| {
                this.show_scrollbar = false;
                cx.notify();
            })
            .ok();
        }))
    }

    fn scroll_to_selected_row(&mut self, _cx: &mut ViewContext<Self>) {
        if let Some(row_ix) = self.selected_row {
            self.vertical_scroll_handle.scroll_to_item(row_ix);
        }
    }

    fn scroll_to_selected_column(&mut self, cx: &mut ViewContext<Self>) {
        if let Some(col_ix) = self.selected_col {
            self.horizontal_scroll_handle.scroll_to_item(col_ix);
            cx.notify();
        }
    }

    fn on_hover_to_autohide_scrollbar(&mut self, hovered: &bool, cx: &mut ViewContext<Self>) {
        if *hovered {
            self.show_scrollbar = true;
            self.hide_scrollbar_task.take();
            cx.notify();
        } else if !self.focus_handle.is_focused(cx) {
            self.hide_scrollbar(cx);
        }
    }

    fn on_row_click(&mut self, row_ix: usize, cx: &mut ViewContext<Self>) {
        self.selection_state = SelectionState::Row;
        self.selected_row = Some(row_ix);
        self.scroll_to_selected_row(cx);
    }

    fn on_col_head_click(&mut self, col_ix: usize, cx: &mut ViewContext<Self>) {
        self.selection_state = SelectionState::Column;
        self.selected_col = Some(col_ix);
        self.scroll_to_selected_column(cx);
    }

    fn action_cancel(&mut self, _: &Cancel, cx: &mut ViewContext<Self>) {
        self.selection_state = SelectionState::Row;
        self.selected_row = None;
        self.selected_col = None;
        cx.notify();
    }

    fn action_select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        let selected_row = self.selected_row.unwrap_or(0);
        let rows_count = self.delegate.rows_count();
        if selected_row > 0 {
            self.selected_row = Some(selected_row - 1);
        } else {
            if self.delegate.can_loop_select() {
                self.selected_row = Some(rows_count - 1);
            }
        }

        self.selection_state = SelectionState::Row;
        self.scroll_to_selected_row(cx);
        cx.notify();
    }

    fn action_select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        let selected_row = self.selected_row.unwrap_or(0);
        if selected_row < self.delegate.rows_count() - 1 {
            self.selected_row = Some(selected_row + 1);
        } else {
            if self.delegate.can_loop_select() {
                self.selected_row = Some(0);
            }
        }

        self.selection_state = SelectionState::Row;
        self.scroll_to_selected_row(cx);
        cx.notify();
    }

    fn action_select_prev_column(&mut self, _: &SelectPrevColumn, cx: &mut ViewContext<Self>) {
        let selected_col = self.selected_col.unwrap_or(0);
        let cols_count = self.delegate.cols_count();
        if selected_col > 0 {
            self.selected_col = Some(selected_col - 1);
        } else {
            if self.delegate.can_loop_select() {
                self.selected_col = Some(cols_count - 1);
            }
        }

        self.selection_state = SelectionState::Column;
        self.scroll_to_selected_column(cx);
        cx.notify();
    }

    fn action_select_next_column(&mut self, _: &SelectNextColumn, cx: &mut ViewContext<Self>) {
        let selected_col = self.selected_col.unwrap_or(0);
        if selected_col < self.delegate.cols_count() - 1 {
            self.selected_col = Some(selected_col + 1);
        } else {
            if self.delegate.can_loop_select() {
                self.selected_col = Some(0);
            }
        }

        self.selection_state = SelectionState::Column;
        self.scroll_to_selected_column(cx);
        cx.notify();
    }

    fn render_cell(&self, col_ix: usize, _cx: &mut ViewContext<Self>) -> Div {
        let col_width = self.col_groups[col_ix].width;

        div()
            .when_some(col_width, |this, width| this.w(px(width)))
            .overflow_hidden()
            .whitespace_nowrap()
            .py_1()
            .px_2()
    }

    /// Show Column selection style, when the column is selected and the selection state is Column.
    fn col_wrap(&self, col_ix: usize, cx: &mut ViewContext<Self>) -> Div {
        if self.selected_col == Some(col_ix) && self.selection_state == SelectionState::Column {
            div().bg(cx.theme().accent.opacity(0.5))
        } else {
            div()
        }
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
        let selected_bg = cx.theme().accent.opacity(0.8);
        let hover_bg = cx.theme().accent.opacity(0.5);

        fn tr(cx: &mut WindowContext) -> Div {
            h_flex().gap_1().border_color(cx.theme().border)
        }

        let inner_table = v_flex()
            .key_context("Table")
            .id("table")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::action_cancel))
            .on_action(cx.listener(Self::action_select_next))
            .on_action(cx.listener(Self::action_select_prev))
            .on_action(cx.listener(Self::action_select_next_column))
            .on_action(cx.listener(Self::action_select_prev_column))
            .on_hover(cx.listener(Self::on_hover_to_autohide_scrollbar))
            .size_full()
            .overflow_hidden()
            .children(self.render_scrollbar(cx))
            .child(
                v_flex()
                    .flex_grow()
                    .h_10()
                    .w_full()
                    .shadow_sm()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        uniform_list(view.clone(), "table-uniform-list-head", 1, {
                            let horizontal_scroll_handle = horizontal_scroll_handle.clone();
                            move |table, _, cx| {
                                // Columns
                                vec![tr(cx)
                                    .id("table-head")
                                    .w_full()
                                    .overflow_scroll()
                                    .track_scroll(&horizontal_scroll_handle)
                                    // The children must be one by one items.
                                    // Becuase the horizontal scroll handle will use the child_item_bounds to
                                    // calculate the item position for itself's `scroll_to_item` method.
                                    .children(table.col_groups.iter().enumerate().map(
                                        |(col_ix, _)| {
                                            table.col_wrap(col_ix, cx).child(
                                                table
                                                    .render_cell(col_ix, cx)
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, _, cx| {
                                                            this.on_col_head_click(col_ix, cx);
                                                        }),
                                                    )
                                                    .child(table.delegate.render_th(col_ix)),
                                            )
                                        },
                                    ))
                                    .flex_1()]
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
                                            table.col_wrap(col_ix, cx).child(
                                                table
                                                    .render_cell(col_ix, cx)
                                                    .flex_shrink_0()
                                                    // Make the row scroll sync with the horizontal_scroll_handle to support horizontal scrolling.
                                                    .left(horizontal_scroll_handle.offset().x)
                                                    .child(
                                                        table.delegate.render_td(row_ix, col_ix),
                                                    ),
                                            )
                                        }))
                                        .when(row_ix > 0, |this| this.border_t_1())
                                        .hover(|this| {
                                            if table.selected_row.is_some() {
                                                this
                                            } else {
                                                this.bg(hover_bg)
                                            }
                                        })
                                        // Row selected style
                                        .when_some(table.selected_row, |this, selected_row| {
                                            this.when(
                                                row_ix == selected_row
                                                    && table.selection_state == SelectionState::Row,
                                                |this| this.bg(selected_bg),
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
            .bg(cx.theme().card)
            .child(inner_table)
            .child(ScrollableMask::new(
                cx.view().clone(),
                ScrollAxis::Horizontal,
                &horizontal_scroll_handle,
            ))
    }
}
