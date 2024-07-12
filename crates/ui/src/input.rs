use std::ops::Range;

use crate::event::InterativeElementExt as _;
use crate::theme::ActiveTheme;
use crate::StyledExt as _;
use blink_cursor::BlinkCursor;
use gpui::*;
use prelude::FluentBuilder as _;
use unicode_segmentation::*;

mod blink_cursor;

actions!(
    input,
    [
        Backspace,
        Delete,
        Enter,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Copy,
        Cut,
        Paste,
        MoveToStartOfLine,
        MoveToEndOfLine,
        TextChanged,
    ]
);

pub enum TextEvent {
    Input { text: SharedString },
    PressEnter,
}

pub fn init(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("enter", Enter, None),
        KeyBinding::new("left", Left, None),
        KeyBinding::new("right", Right, None),
        KeyBinding::new("shift-left", SelectLeft, None),
        KeyBinding::new("shift-right", SelectRight, None),
        KeyBinding::new("home", Home, None),
        KeyBinding::new("end", End, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-a", SelectAll, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-a", SelectAll, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-c", Copy, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-c", Copy, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-x", Cut, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-x", Cut, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-v", Paste, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-v", Paste, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("ctrl-a", Home, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("ctrl-e", End, None),
    ]);
}

pub struct TextInput {
    focus_handle: FocusHandle,
    text: SharedString,
    prefix: Option<AnyView>,
    suffix: Option<AnyView>,
    placeholder: SharedString,
    blink_cursor: Model<BlinkCursor>,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    disabled: bool,
    masked: bool,
    appearance: bool,
}

impl EventEmitter<TextEvent> for TextInput {}

impl TextInput {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let blink_cursor = cx.new_model(|cx| BlinkCursor::new(cx));
        let input = Self {
            focus_handle: focus_handle.clone(),
            text: "".into(),
            placeholder: "".into(),
            blink_cursor,
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            disabled: false,
            masked: false,
            appearance: true,
            prefix: None,
            suffix: None,
        };

        // Observe the blink cursor to repaint the view when it changes.
        cx.observe(&input.blink_cursor, |_, _, cx| cx.notify())
            .detach();
        // Blink the cursor when the window is active, pause when it's not.
        cx.observe_window_activation(|input, cx| {
            if cx.is_window_active() {
                input.blink_cursor.update(cx, |blink_cursor, cx| {
                    blink_cursor.start(cx);
                });
            }
        })
        .detach();

        cx.on_focus(&focus_handle, Self::on_focus).detach();
        cx.on_blur(&focus_handle, Self::on_blur).detach();

        input
    }

    /// Set the text of the input field.
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut ViewContext<Self>) {
        self.text = text.into();
        self.selected_range = self.text.len()..self.text.len();
        cx.notify();
    }

    /// Set the placeholder text of the input field.
    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
    ) {
        self.placeholder = placeholder.into();
        cx.notify();
    }

    /// Set the disabled state of the input field.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut ViewContext<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    /// Set the masked state of the input field.
    pub fn set_masked(&mut self, masked: bool, cx: &mut ViewContext<Self>) {
        self.masked = masked;
        cx.notify();
    }

    /// Set the appearance of the input field.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    /// Set the prefix element of the input field, for example a search Icon.
    pub fn prefix(mut self, prefix: impl Into<AnyView>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the placeholder text of the input field.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the suffix element of the input field, for example a clear button.
    pub fn suffix(mut self, suffix: impl Into<AnyView>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Return the text of the input field.
    pub fn text(&self) -> SharedString {
        self.text.clone()
    }

    fn left(&mut self, _: &Left, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx)
        }
    }

    fn right(&mut self, _: &Right, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, cx: &mut ViewContext<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, cx: &mut ViewContext<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, cx: &mut ViewContext<Self>) {
        self.move_to(0, cx);
        self.select_to(self.text.len(), cx)
    }

    fn home(&mut self, _: &Home, cx: &mut ViewContext<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, cx: &mut ViewContext<Self>) {
        self.move_to(self.text.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", cx)
    }

    fn delete(&mut self, _: &Delete, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", cx)
    }

    fn enter(&mut self, _: &Enter, cx: &mut ViewContext<Self>) {
        cx.emit(TextEvent::PressEnter);
    }

    fn show_character_palette(&mut self, _: &ShowCharacterPalette, cx: &mut ViewContext<Self>) {
        cx.show_character_palette();
    }

    fn copy(&mut self, _: &Copy, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            return;
        }

        let selected_text = self.text[self.selected_range.clone()].to_string();
        cx.write_to_clipboard(ClipboardItem::new(selected_text));
    }

    fn cut(&mut self, _: &Cut, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            return;
        }

        let selected_text = self.text[self.selected_range.clone()].to_string();
        cx.write_to_clipboard(ClipboardItem::new(selected_text));
        self.replace_text_in_range(Some(self.selected_range.clone()), "", cx);
    }

    pub fn paste(&mut self, _: &Paste, cx: &mut ViewContext<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let new_text = clipboard.text().replace('\n', "");
            self.replace_text_in_range(Some(self.selected_range.clone()), &new_text, cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        self.selected_range = offset..offset;
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn select_to(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.text.len())
    }

    /// Returns the true to let InputElement to render cursor, when Input is focused and current BlinkCursor is visible.
    pub(crate) fn show_cursor(&self, cx: &WindowContext) -> bool {
        self.focus_handle.is_focused(cx) && self.blink_cursor.read(cx).visible()
    }

    fn on_focus(&mut self, cx: &mut ViewContext<Self>) {
        self.blink_cursor.update(cx, |blink_cursor, cx| {
            blink_cursor.start(cx);
        });
    }

    fn on_blur(&mut self, cx: &mut ViewContext<Self>) {
        self.blink_cursor.update(cx, |blink_cursor, cx| {
            blink_cursor.pause(cx);
        });
    }

    fn on_mouse_left_down(
        &mut self,
        event: &MouseDownEvent,
        text_hitbox: Hitbox,
        cx: &mut ViewContext<TextInput>,
    ) {
        if !text_hitbox.contains(&event.position) {
            return;
        }

        let position = event.position - text_hitbox.origin;
        let offset = self
            .last_layout
            .as_ref()
            .and_then(|layout| layout.index_for_x(position.x));
        if offset.is_none() {
            return;
        }

        let offset = offset.unwrap();

        // If shift present
        if event.modifiers.shift {
            // Select to
            self.select_to(offset, cx);
        } else {
            // Move to
            self.move_to(offset, cx);
        }
    }
}

impl ViewInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        Some(self.text[range].to_string())
    }

    fn selected_text_range(&mut self, _cx: &mut ViewContext<Self>) -> Option<Range<usize>> {
        Some(self.range_to_utf16(&self.selected_range))
    }

    fn marked_text_range(&self, _cx: &mut ViewContext<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _cx: &mut ViewContext<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.text =
            (self.text[0..range.start].to_owned() + new_text + &self.text[range.end..]).into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.emit(TextEvent::Input {
            text: self.text.clone(),
        });
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        cx: &mut ViewContext<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.text =
            (self.text[0..range.start].to_owned() + new_text + &self.text[range.end..]).into();
        self.marked_range = Some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        cx.emit(TextEvent::Input {
            text: self.text.clone(),
        });
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }
}

impl FocusableView for TextInput {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct TextElement {
    input: View<TextInput>,
}

impl TextElement {
    fn paint_mouse_listeners(&mut self, hitbox: &Hitbox, cx: &mut WindowContext) {
        let input = self.input.clone();
        let hitbox = hitbox.clone();

        cx.on_mouse_event(move |event: &MouseDownEvent, phase, cx| {
            if phase == DispatchPhase::Bubble {
                match event.button {
                    MouseButton::Left => {
                        let hitbox = hitbox.clone();
                        input.update(cx, |input, cx| {
                            input.on_mouse_left_down(event, hitbox, cx);
                        });
                    }
                    _ => {}
                }
            }
        });
    }
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    hitbox: Hitbox,
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
        style.size.height = cx.line_height().into();
        (cx.request_layout(style, []), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let text = input.text.clone();
        let placeholder = input.placeholder.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = cx.text_style();

        let (disaplay_text, text_color) = if text.is_empty() {
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
            len: input.text.len(),
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
                    len: disaplay_text.len() - marked_range.end,
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
        let line = cx
            .text_system()
            .shape_line(disaplay_text, font_size, &runs)
            .unwrap();

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() && input.show_cursor(cx) {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(1.5), bounds.bottom() - bounds.top()),
                    ),
                    gpui::blue(),
                )),
            )
        } else {
            (
                Some(fill(
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
                )),
                None,
            )
        };

        let hitbox = cx.insert_hitbox(bounds, false);

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
            hitbox,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        let focused = focus_handle.is_focused(cx);

        cx.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
        );
        if let Some(selection) = prepaint.selection.take() {
            cx.paint_quad(selection)
        }
        let line = prepaint.line.take().unwrap();
        line.paint(bounds.origin, cx.line_height(), cx).unwrap();

        if focused {
            if let Some(cursor) = prepaint.cursor.take() {
                cx.paint_quad(cursor);
            }
        }
        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
        });
        self.paint_mouse_listeners(&prepaint.hitbox, cx);
    }
}

impl Render for TextInput {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(cx);

        div()
            .flex()
            .key_context("TextInput")
            .track_focus(&self.focus_handle)
            .when(!self.disabled, |this| {
                this.on_action(cx.listener(Self::backspace))
                    .on_action(cx.listener(Self::delete))
                    .on_action(cx.listener(Self::enter))
            })
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            // Double click to select all
            .on_double_click(cx.listener(|view, _, cx| {
                view.select_all(&SelectAll, cx);
            }))
            .size_full()
            .line_height(rems(1.25))
            .text_size(rems(0.875))
            .py_2()
            .h_10()
            .when(self.appearance, |this| {
                this.bg(cx.theme().input)
                    .border_color(cx.theme().input)
                    .border_1()
                    .rounded(px(cx.theme().radius))
                    .shadow_sm()
                    .when(focused, |this| this.outline(cx))
                    .px_3()
                    .bg(if self.disabled {
                        cx.theme().muted
                    } else {
                        cx.theme().background
                    })
            })
            .when_some(self.prefix.clone(), |this, prefix| this.child(prefix))
            .gap_1()
            .items_center()
            .child(div().flex_grow().overflow_x_hidden().child(TextElement {
                input: cx.view().clone(),
            }))
            .when_some(self.suffix.clone(), |this, suffix| this.child(suffix))
    }
}
