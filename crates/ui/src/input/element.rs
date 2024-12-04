use gpui::{
    fill, point, px, relative, size, Bounds, Element, ElementId, ElementInputHandler,
    GlobalElementId, IntoElement, LayoutId, MouseButton, MouseMoveEvent, PaintQuad, Pixels, Point,
    Style, TextRun, UnderlineStyle, View, WindowContext, WrappedLine,
};
use smallvec::SmallVec;

use crate::theme::ActiveTheme as _;

use super::TextInput;

pub(super) struct TextElement {
    input: View<TextInput>,
}

impl TextElement {
    pub(super) fn new(input: View<TextInput>) -> Self {
        Self { input }
    }

    fn paint_mouse_listeners(&mut self, cx: &mut WindowContext) {
        cx.on_mouse_event({
            let input = self.input.clone();

            move |event: &MouseMoveEvent, _, cx| {
                if event.pressed_button == Some(MouseButton::Left) {
                    input.update(cx, |input, cx| {
                        input.on_drag_move(event, cx);
                    });
                }
            }
        });
    }
}

pub(super) struct PrepaintState {
    scroll_offset: Point<Pixels>,
    lines: SmallVec<[WrappedLine; 1]>,
    cursor: Option<PaintQuad>,
    selections: SmallVec<[PaintQuad; 1]>,
    bounds: Bounds<Pixels>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = if self.input.read(cx).multi_line {
            (10. * cx.line_height()).into()
        } else {
            cx.line_height().into()
        };
        (cx.request_layout(style, []), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        let multi_line = self.input.read(cx).multi_line;
        let line_height = cx.line_height();
        let input = self.input.read(cx);
        let text = input.text.clone();
        let placeholder = input.placeholder.clone();
        let selected_range = input.selected_range.clone();
        let cursor_offset = input.cursor_offset();
        let style = cx.text_style();

        let (display_text, text_color) = if text.is_empty() {
            (placeholder, cx.theme().muted_foreground)
        } else if input.masked {
            (
                "*".repeat(text.chars().count()).into(),
                cx.theme().foreground,
            )
        } else {
            (text, cx.theme().foreground)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run.clone()
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(cx.rem_size());
        let wrap_width = if multi_line {
            Some(bounds.size.width)
        } else {
            None
        };

        let lines = cx
            .text_system()
            .shape_text(display_text, font_size, &runs, wrap_width)
            .unwrap();

        // Calculate the scroll offset to keep the cursor in view
        let mut scroll_offset = input.scroll_offset;
        let mut bounds = bounds;
        let mut selections = SmallVec::new();
        let mut cursor = None;
        const RIGHT_MARGIN: Pixels = px(5.);
        const INSET: Pixels = px(0.5);

        // layout_visible_cursors
        let mut cursor_pos = None;
        let mut cursor_start = None;
        let mut cursor_end = None;
        let has_selection = !selected_range.is_empty();

        for (ix, line) in lines.iter().enumerate() {
            // break loop if all cursor positions are found
            if cursor_pos.is_some() && cursor_start.is_some() && cursor_end.is_some() {
                break;
            }

            if cursor_pos.is_none() {
                cursor_pos = line.position_for_index(cursor_offset, line_height);
            }

            let line_cursor_start = line.position_for_index(selected_range.start, line_height);
            if cursor_start.is_none() {
                cursor_start = line_cursor_start;
            }
            let line_cursor_end = line.position_for_index(selected_range.end, line_height);
            if cursor_end.is_none() {
                cursor_end = line_cursor_end;
            }
        }

        if let (Some(cursor_pos), Some(cursor_start), Some(cursor_end)) =
            (cursor_pos, cursor_start, cursor_end)
        {
            let cursor_moved = input.last_cursor_offset != Some(cursor_offset);
            let selection_changed = input.last_selected_range != Some(selected_range.clone());

            if cursor_moved || selection_changed {
                scroll_offset.x =
                    if scroll_offset.x + cursor_pos.x > (bounds.size.width - RIGHT_MARGIN) {
                        // cursor is out of right
                        bounds.size.width - RIGHT_MARGIN - cursor_pos.x
                    } else if scroll_offset.x + cursor_pos.x < px(0.) {
                        // cursor is out of left
                        scroll_offset.x - cursor_pos.x
                    } else {
                        scroll_offset.x
                    };
                scroll_offset.y = if scroll_offset.y + cursor_pos.y > (bounds.size.height) {
                    // cursor is out of bottom
                    bounds.size.height - cursor_pos.y
                } else if scroll_offset.y + cursor_pos.y < px(0.) {
                    // cursor is out of top
                    scroll_offset.y - cursor_pos.y
                } else {
                    scroll_offset.y
                };

                if input.selection_reversed {
                    if scroll_offset.x + cursor_start.x < px(0.) {
                        // selection start is out of left
                        scroll_offset.x = -cursor_start.x;
                    }
                    if scroll_offset.y + cursor_start.y < px(0.) {
                        // selection start is out of top
                        scroll_offset.y = -cursor_start.y;
                    }
                } else {
                    if scroll_offset.x + cursor_end.x <= px(0.) {
                        // selection end is out of left
                        scroll_offset.x = -cursor_end.x;
                    }
                    if scroll_offset.y + cursor_end.y <= px(0.) {
                        // selection end is out of top
                        scroll_offset.y = -cursor_end.y;
                    }
                }
            }

            bounds.origin = bounds.origin + scroll_offset;

            if input.show_cursor(cx) {
                // cursor blink
                cursor = Some(fill(
                    Bounds::new(
                        point(
                            bounds.left() + cursor_pos.x,
                            bounds.top() + cursor_pos.y + INSET,
                        ),
                        size(px(2.), line_height),
                    ),
                    crate::blue_500(),
                ))
            };
        }

        // layout selections
        if has_selection {
            for (ix, line) in lines.iter().enumerate() {
                // selections background for each lines
                if cursor_start.is_some() || cursor_end.is_some() {
                    let start = cursor_start
                        .unwrap_or_else(|| line.position_for_index(0, line_height).unwrap());
                    let end = cursor_end.unwrap_or_else(|| {
                        line.position_for_index(line.len(), line_height).unwrap()
                    });

                    let selection = fill(
                        Bounds::from_corners(
                            point(
                                bounds.left() + start.x,
                                bounds.top() + ix as f32 * line_height,
                            ),
                            point(
                                bounds.left() + end.x,
                                bounds.top() + (ix + 1) as f32 * line_height,
                            ),
                        ),
                        cx.theme().selection,
                    );
                    selections.push(selection);
                }
            }
        }

        PrepaintState {
            scroll_offset,
            bounds,
            lines,
            cursor,
            selections,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        let focused = focus_handle.is_focused(cx);
        let bounds = prepaint.bounds;
        let selected_range = self.input.read(cx).selected_range.clone();
        let cursor = self.input.read(cx).cursor_offset();

        cx.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
        );

        // Paint selections
        for selection in prepaint.selections.iter() {
            cx.paint_quad(selection.clone());
        }

        // // Paint single line text
        // if let Some(line) = prepaint.line.take() {
        //     line.paint(bounds.origin, cx.line_height(), cx).unwrap();
        //     last_layout = Some(line);
        // }

        // Paint multi line text
        let line_height = cx.line_height();
        let origin = bounds.origin;

        for (i, line) in prepaint.lines.iter().enumerate() {
            let p = point(origin.x, origin.y + i as f32 * line_height);
            _ = line.paint(p, line_height, cx);
        }

        if focused {
            if let Some(cursor) = prepaint.cursor.take() {
                cx.paint_quad(cursor);
            }
        }
        self.input.update(cx, |input, _cx| {
            input.scroll_offset = prepaint.scroll_offset;
            input.last_layout = Some(prepaint.lines.clone());
            input.last_bounds = Some(bounds);
            input.last_cursor_offset = Some(cursor);
            input.last_selected_range = Some(selected_range);
        });

        self.paint_mouse_listeners(cx);
    }
}
