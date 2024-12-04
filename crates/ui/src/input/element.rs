use gpui::{
    fill, point, px, relative, size, Bounds, Element, ElementId, ElementInputHandler,
    GlobalElementId, IntoElement, LayoutId, MouseButton, MouseMoveEvent, PaintQuad, Pixels, Point,
    ShapedLine, Style, TextRun, UnderlineStyle, View, WindowContext, WrappedLine,
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
    line: Option<ShapedLine>,
    lines: Option<SmallVec<[WrappedLine; 1]>>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
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
        let mut line = None;
        let mut lines = None;
        if multi_line {
            let wrap_width = bounds.size.width;
            lines = cx
                .text_system()
                .shape_text(display_text, font_size, &runs, Some(wrap_width))
                .ok();
        } else {
            line = cx
                .text_system()
                .shape_line(display_text, font_size, &runs)
                .ok()
        };

        // Calculate the scroll offset to keep the cursor in view
        let mut scroll_offset = input.scroll_offset;
        let mut bounds = bounds;
        let mut selection = None;
        let mut cursor = None;
        const RIGHT_MARGIN: Pixels = px(5.);
        const INSET: Pixels = px(0.5);

        if let Some(line) = &line {
            let cursor_pos = line.x_for_index(cursor_offset);
            let cursor_start = line.x_for_index(selected_range.start);
            let cursor_end = line.x_for_index(selected_range.end);
            let cursor_moved = input.last_cursor_offset != Some(cursor_offset);
            let selection_changed = input.last_selected_range != Some(selected_range.clone());

            if cursor_moved || selection_changed {
                scroll_offset.x =
                    if scroll_offset.x + cursor_pos > (bounds.size.width - RIGHT_MARGIN) {
                        // cursor is out of right
                        bounds.size.width - RIGHT_MARGIN - cursor_pos
                    } else if scroll_offset.x + cursor_pos < px(0.) {
                        // cursor is out of left
                        scroll_offset.x - cursor_pos
                    } else {
                        scroll_offset.x
                    };

                if input.selection_reversed {
                    if scroll_offset.x + cursor_start < px(0.) {
                        // selection start is out of left
                        scroll_offset.x = -cursor_start;
                    }
                } else {
                    if scroll_offset.x + cursor_end <= px(0.) {
                        // selection end is out of left
                        scroll_offset.x = -cursor_end;
                    }
                }
            }

            bounds.origin = bounds.origin + scroll_offset;

            if selected_range.is_empty() && input.show_cursor(cx) {
                // cursor blink
                cursor = Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top() + INSET),
                        size(px(2.), bounds.bottom() - bounds.top() - INSET * 2),
                    ),
                    crate::blue_500(),
                ))
            } else {
                // selection background
                selection = Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    cx.theme().selection,
                ))
            };
        }

        PrepaintState {
            scroll_offset,
            bounds,
            line,
            lines,
            cursor,
            selection,
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
        if let Some(selection) = prepaint.selection.take() {
            cx.paint_quad(selection)
        }

        let mut last_layout = None;

        // Paint single line text
        if let Some(line) = prepaint.line.take() {
            line.paint(bounds.origin, cx.line_height(), cx).unwrap();
            last_layout = Some(line);
        }

        // Paint multi line text
        if let Some(lines) = prepaint.lines.take() {
            let line_height = cx.line_height();
            let origin = bounds.origin;

            for (i, line) in lines.into_iter().enumerate() {
                let p = point(origin.x, origin.y + i as f32 * line_height);
                line.paint(p, line_height, cx).unwrap();
            }
        }

        if focused {
            if let Some(cursor) = prepaint.cursor.take() {
                cx.paint_quad(cursor);
            }
        }
        self.input.update(cx, |input, _cx| {
            input.scroll_offset = prepaint.scroll_offset;
            // FIXME: To support multi-line text, we need to store the last layout for each line.
            input.last_layout = last_layout;
            input.last_bounds = Some(bounds);
            input.last_cursor_offset = Some(cursor);
            input.last_selected_range = Some(selected_range);
        });

        self.paint_mouse_listeners(cx);
    }
}
