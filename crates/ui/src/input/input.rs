//! A text input field that allows the user to enter text.
//!
//! Based on the `Input` example from the `gpui` crate.
//! https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/input.rs

use smallvec::SmallVec;
use std::ops::Range;
use unicode_segmentation::*;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    actions, div, point, px, AnyElement, AppContext, Bounds, ClickEvent, ClipboardItem,
    Context as _, EventEmitter, FocusHandle, FocusableView, InteractiveElement as _, IntoElement,
    KeyBinding, KeyDownEvent, Model, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement as _, Pixels, Point, Rems, Render, SharedString, Styled as _, UTF16Selection,
    ViewContext, ViewInputHandler, WindowContext, WrappedLine,
};

// TODO:
// - Press Up,Down to move cursor up, down line if multi-line
// - Move cursor to skip line eof empty chars.

use super::blink_cursor::BlinkCursor;
use super::change::Change;
use super::element::TextElement;
use super::ClearButton;

use crate::history::History;
use crate::indicator::Indicator;
use crate::theme::ActiveTheme;
use crate::StyledExt as _;
use crate::{event::InteractiveElementExt as _, Size};
use crate::{Sizable, StyleSized};

actions!(
    input,
    [
        Backspace,
        Delete,
        Enter,
        Up,
        Down,
        Left,
        Right,
        SelectUp,
        SelectDown,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        SelectToHome,
        SelectToEnd,
        ShowCharacterPalette,
        Copy,
        Cut,
        Paste,
        Undo,
        Redo,
        MoveToStartOfLine,
        MoveToEndOfLine,
        TextChanged,
    ]
);

#[derive(Clone)]
pub enum InputEvent {
    Change(SharedString),
    PressEnter,
    Focus,
    Blur,
}

const CONTEXT: &str = "Input";

pub fn init(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some(CONTEXT)),
        KeyBinding::new("delete", Delete, Some(CONTEXT)),
        KeyBinding::new("enter", Enter, Some(CONTEXT)),
        KeyBinding::new("left", Left, Some(CONTEXT)),
        KeyBinding::new("right", Right, Some(CONTEXT)),
        KeyBinding::new("shift-left", SelectLeft, Some(CONTEXT)),
        KeyBinding::new("shift-right", SelectRight, Some(CONTEXT)),
        KeyBinding::new("up", Up, Some(CONTEXT)),
        KeyBinding::new("right", Down, Some(CONTEXT)),
        KeyBinding::new("shift-up", SelectUp, Some(CONTEXT)),
        KeyBinding::new("shift-down", SelectDown, Some(CONTEXT)),
        KeyBinding::new("home", Home, Some(CONTEXT)),
        KeyBinding::new("end", End, Some(CONTEXT)),
        KeyBinding::new("shift-home", SelectToHome, Some(CONTEXT)),
        KeyBinding::new("shift-end", SelectToEnd, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("shift-cmd-left", SelectToHome, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("shift-cmd-right", SelectToEnd, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-a", SelectAll, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-a", SelectAll, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-c", Copy, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-c", Copy, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-x", Cut, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-x", Cut, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-v", Paste, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-v", Paste, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("ctrl-a", Home, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-left", Home, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("ctrl-e", End, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-right", End, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-z", Undo, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-shift-z", Redo, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-z", Undo, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-y", Redo, Some(CONTEXT)),
    ]);
}

pub struct TextInput {
    pub(super) focus_handle: FocusHandle,
    pub(super) text: SharedString,
    pub(super) multi_line: bool,
    pub(super) history: History<Change>,
    pub(super) blink_cursor: Model<BlinkCursor>,
    pub(super) prefix: Option<Box<dyn Fn(&mut ViewContext<Self>) -> AnyElement + 'static>>,
    pub(super) suffix: Option<Box<dyn Fn(&mut ViewContext<Self>) -> AnyElement + 'static>>,
    pub(super) loading: bool,
    pub(super) placeholder: SharedString,
    pub(super) selected_range: Range<usize>,
    pub(super) selection_reversed: bool,
    pub(super) marked_range: Option<Range<usize>>,
    pub(super) last_layout: Option<SmallVec<[WrappedLine; 1]>>,
    pub(super) last_bounds: Option<Bounds<Pixels>>,
    pub(super) last_cursor_offset: Option<usize>,
    pub(super) last_selected_range: Option<Range<usize>>,
    pub(super) scroll_offset: Point<Pixels>,
    pub(super) is_selecting: bool,
    pub(super) disabled: bool,
    pub(super) masked: bool,
    pub(super) appearance: bool,
    pub(super) cleanable: bool,
    pub(super) size: Size,
    pattern: Option<regex::Regex>,
    validate: Option<Box<dyn Fn(&str) -> bool + 'static>>,
}

impl EventEmitter<InputEvent> for TextInput {}

impl TextInput {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let blink_cursor = cx.new_model(|_| BlinkCursor::new());
        let history = History::new().group_interval(std::time::Duration::from_secs(1));
        let input = Self {
            focus_handle: focus_handle.clone(),
            text: "".into(),
            multi_line: false,
            blink_cursor,
            history,
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            last_cursor_offset: None,
            last_selected_range: None,
            scroll_offset: point(px(0.), px(0.)),
            is_selecting: false,
            disabled: false,
            masked: false,
            appearance: true,
            cleanable: false,
            loading: false,
            prefix: None,
            suffix: None,
            size: Size::Medium,
            pattern: None,
            validate: None,
        };

        // Observe the blink cursor to repaint the view when it changes.
        cx.observe(&input.blink_cursor, |_, _, cx| cx.notify())
            .detach();
        // Blink the cursor when the window is active, pause when it's not.
        cx.observe_window_activation(|input, cx| {
            if cx.is_window_active() {
                let focus_handle = input.focus_handle.clone();
                if focus_handle.is_focused(cx) {
                    input.blink_cursor.update(cx, |blink_cursor, cx| {
                        blink_cursor.start(cx);
                    });
                }
            }
        })
        .detach();

        cx.on_focus(&focus_handle, Self::on_focus).detach();
        cx.on_blur(&focus_handle, Self::on_blur).detach();

        input
    }

    /// Use the text input field as a multi-line Textarea.
    pub fn multi_line(mut self) -> Self {
        self.multi_line = true;
        self
    }

    /// Set the text of the input field.
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut ViewContext<Self>) {
        self.history.ignore = true;
        self.replace_text(text, cx);
        self.history.ignore = false;

        cx.notify();
    }

    fn replace_text(&mut self, text: impl Into<SharedString>, cx: &mut ViewContext<Self>) {
        let text: SharedString = text.into();
        let range = 0..self.text.chars().map(|c| c.len_utf16()).sum();
        self.replace_text_in_range(Some(range), &text, cx);
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

    /// Set the prefix element of the input field.
    pub fn set_prefix<F, E>(&mut self, builder: F, cx: &mut ViewContext<Self>)
    where
        F: Fn(&ViewContext<Self>) -> E + 'static,
        E: IntoElement,
    {
        self.prefix = Some(Box::new(move |cx| builder(cx).into_any_element()));
        cx.notify();
    }

    /// Set the suffix element of the input field.
    pub fn set_suffix<F, E>(&mut self, builder: F, cx: &mut ViewContext<Self>)
    where
        F: Fn(&ViewContext<Self>) -> E + 'static,
        E: IntoElement,
    {
        self.suffix = Some(Box::new(move |cx| builder(cx).into_any_element()));
        cx.notify();
    }

    /// Set the Input size
    pub fn set_size(&mut self, size: Size, cx: &mut ViewContext<Self>) {
        self.size = size;
        cx.notify();
    }

    /// Set the appearance of the input field.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    /// Set the prefix element of the input field, for example a search Icon.
    pub fn prefix<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut ViewContext<Self>) -> E + 'static,
        E: IntoElement,
    {
        self.prefix = Some(Box::new(move |cx| builder(cx).into_any_element()));
        self
    }

    /// Set the suffix element of the input field, for example a clear button.
    pub fn suffix<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut ViewContext<Self>) -> E + 'static,
        E: IntoElement,
    {
        self.suffix = Some(Box::new(move |cx| builder(cx).into_any_element()));
        self
    }

    /// Set the placeholder text of the input field.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the placeholder text of the input field with reference.
    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>) {
        self.placeholder = placeholder.into();
    }

    /// Set true to show the clear button when the input field is not empty.
    pub fn cleanable(mut self) -> Self {
        self.cleanable = true;
        self
    }

    /// Set the regular expression pattern of the input field.
    pub fn pattern(mut self, pattern: regex::Regex) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Set the regular expression pattern of the input field with reference.
    pub fn set_pattern(&mut self, pattern: regex::Regex) {
        self.pattern = Some(pattern);
    }

    /// Set the validation function of the input field.
    pub fn validate(mut self, f: impl Fn(&str) -> bool + 'static) -> Self {
        self.validate = Some(Box::new(f));
        self
    }

    /// Set true to show indicator at the input right.
    pub fn set_loading(&mut self, loading: bool, cx: &mut ViewContext<Self>) {
        self.loading = loading;
        cx.notify();
    }

    /// Return the text of the input field.
    pub fn text(&self) -> SharedString {
        self.text.clone()
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// Focus the input field.
    pub fn focus(&self, cx: &mut ViewContext<Self>) {
        self.focus_handle.focus(cx);
    }

    fn left(&mut self, _: &Left, cx: &mut ViewContext<Self>) {
        self.pause_blink_cursor(cx);
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx)
        }
    }

    fn right(&mut self, _: &Right, cx: &mut ViewContext<Self>) {
        self.pause_blink_cursor(cx);
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    fn select_up(&mut self, _: &SelectLeft, cx: &mut ViewContext<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
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
        self.pause_blink_cursor(cx);
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, cx: &mut ViewContext<Self>) {
        self.pause_blink_cursor(cx);
        self.move_to(self.text.len(), cx);
    }

    fn select_to_home(&mut self, _: &SelectToHome, cx: &mut ViewContext<Self>) {
        self.select_to(0, cx);
    }

    fn select_to_end(&mut self, _: &SelectToEnd, cx: &mut ViewContext<Self>) {
        self.select_to(self.text.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", cx);
        self.pause_blink_cursor(cx);
    }

    fn delete(&mut self, _: &Delete, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", cx);
        self.pause_blink_cursor(cx);
    }

    fn enter(&mut self, _: &Enter, cx: &mut ViewContext<Self>) {
        if self.multi_line {
            self.replace_text_in_range(None, "\n", cx);
            // Move cursor to the start of the next line
            // TODO: To be test this line is valid
            self.move_to(self.next_boundary(self.cursor_offset()), cx);
        }

        cx.emit(InputEvent::PressEnter);
    }

    fn clean(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.replace_text("", cx);
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut ViewContext<Self>) {
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position, cx);

        // Double click to select word
        if event.button == MouseButton::Left && event.click_count == 2 {
            self.select_word(offset, cx);
            return;
        }

        if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut ViewContext<Self>) {
        self.is_selecting = false;
    }

    fn show_character_palette(&mut self, _: &ShowCharacterPalette, cx: &mut ViewContext<Self>) {
        cx.show_character_palette();
    }

    fn copy(&mut self, _: &Copy, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            return;
        }

        let selected_text = self.text[self.selected_range.clone()].to_string();
        cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
    }

    fn cut(&mut self, _: &Cut, cx: &mut ViewContext<Self>) {
        if self.selected_range.is_empty() {
            return;
        }

        let selected_text = self.text[self.selected_range.clone()].to_string();
        cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
        self.replace_text_in_range(None, "", cx);
    }

    fn paste(&mut self, _: &Paste, cx: &mut ViewContext<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let mut new_text = clipboard.text().unwrap_or_default();
            if !self.multi_line {
                new_text = new_text.replace('\n', "");
            }

            self.replace_text_in_range(None, &new_text, cx);
        }
    }

    fn push_history(&mut self, range: &Range<usize>, new_text: &str, cx: &mut ViewContext<Self>) {
        if self.history.ignore {
            return;
        }

        let old_text = self
            .text_for_range(self.range_to_utf16(&range), &mut None, cx)
            .unwrap_or("".to_string());

        let new_range = range.start..range.start + new_text.len();

        self.history.push(Change::new(
            range.clone(),
            &old_text,
            new_range.clone(),
            new_text,
        ));
    }

    fn undo(&mut self, _: &Undo, cx: &mut ViewContext<Self>) {
        self.history.ignore = true;
        if let Some(changes) = self.history.undo() {
            for change in changes {
                let range_utf16 = self.range_to_utf16(&change.new_range);
                self.replace_text_in_range(Some(range_utf16), &change.old_text, cx);
            }
        }
        self.history.ignore = false;
    }

    fn redo(&mut self, _: &Redo, cx: &mut ViewContext<Self>) {
        self.history.ignore = true;
        if let Some(changes) = self.history.redo() {
            for change in changes {
                let range_utf16 = self.range_to_utf16(&change.old_range);
                self.replace_text_in_range(Some(range_utf16), &change.new_text, cx);
            }
        }
        self.history.ignore = false;
    }

    fn move_to(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        self.selected_range = offset..offset;
        self.pause_blink_cursor(cx);
        cx.notify()
    }

    pub(super) fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>, cx: &WindowContext) -> usize {
        let line_height = cx.line_height();

        // If the text is empty, always return 0
        if self.text.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(lines)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };

        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.text.len();
        }

        let mut ix = 0;
        for line in lines {
            if let Ok(index) = line.index_for_position(position - bounds.origin, line_height) {
                ix += index;
                break;
            } else {
                ix += line.len()
            }
        }

        ix
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

    /// Select the word at the given offset.
    fn select_word(&mut self, offset: usize, cx: &mut ViewContext<Self>) {
        fn is_word(c: char) -> bool {
            c.is_alphanumeric() || matches!(c, '_')
        }

        let mut start = self.offset_to_utf16(offset);
        let mut end = start;
        let prev_text = self
            .text_for_range(0..start, &mut None, cx)
            .unwrap_or_default();
        let next_text = self
            .text_for_range(end..self.text.len(), &mut None, cx)
            .unwrap_or_default();

        let prev_chars = prev_text.chars().rev().peekable();
        let next_chars = next_text.chars().peekable();

        for (_, c) in prev_chars.enumerate() {
            if !is_word(c) {
                break;
            }

            start -= c.len_utf16();
        }

        for (_, c) in next_chars.enumerate() {
            if !is_word(c) {
                break;
            }

            end += c.len_utf16();
        }

        self.selected_range = self.range_from_utf16(&(start..end));
        cx.notify()
    }

    fn unselect(&mut self, cx: &mut ViewContext<Self>) {
        self.selected_range = self.cursor_offset()..self.cursor_offset();
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
        self.blink_cursor.update(cx, |cursor, cx| {
            cursor.start(cx);
        });
        cx.emit(InputEvent::Focus);
    }

    fn on_blur(&mut self, cx: &mut ViewContext<Self>) {
        self.unselect(cx);
        self.blink_cursor.update(cx, |cursor, cx| {
            cursor.stop(cx);
        });
        cx.emit(InputEvent::Blur);
    }

    fn pause_blink_cursor(&mut self, cx: &mut ViewContext<Self>) {
        self.blink_cursor.update(cx, |cursor, cx| {
            cursor.pause(cx);
        });
    }

    fn on_key_down_for_blink_cursor(&mut self, _: &KeyDownEvent, cx: &mut ViewContext<Self>) {
        self.pause_blink_cursor(cx)
    }

    pub(super) fn on_drag_move(&mut self, event: &MouseMoveEvent, cx: &mut ViewContext<Self>) {
        if self.text.is_empty() {
            return;
        }

        if self.last_layout.is_none() {
            return;
        }

        if !self.focus_handle.is_focused(cx) {
            return;
        }

        if !self.is_selecting {
            return;
        }

        let offset = self.offset_of_position(event.position, cx);
        self.select_to(offset, cx);
    }

    fn offset_of_position(&self, position: Point<Pixels>, cx: &WindowContext) -> usize {
        let line_height = cx.line_height();
        let bounds = self.last_bounds.unwrap_or_default();
        let position = position - bounds.origin;

        let Some(lines) = self.last_layout.as_ref() else {
            return 0;
        };

        for line in lines {
            if let Ok(index) = line.index_for_position(position, line_height) {
                return index;
            }
        }

        // If the mouse is on the right side of the last character, move to the end
        // Otherwise, move to the start of the line
        let last_line = lines.last().unwrap();
        let last_index = last_line.len();
        if let Some(last_x) = last_line.position_for_index(last_index, line_height) {
            if position.x > last_x.x {
                return last_index;
            }
        }

        0
    }

    fn is_valid_input(&self, new_text: &str) -> bool {
        if new_text.is_empty() {
            return true;
        }

        if let Some(validate) = &self.validate {
            if !validate(new_text) {
                return false;
            }
        }

        self.pattern
            .as_ref()
            .map(|p| p.is_match(new_text))
            .unwrap_or(true)
    }
}

impl Sizable for TextInput {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl ViewInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _cx: &mut ViewContext<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        adjusted_range.replace(self.range_to_utf16(&range));
        Some(self.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _cx: &mut ViewContext<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: false,
        })
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
        if self.disabled {
            return;
        }

        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let pending_text: SharedString =
            (self.text[0..range.start].to_owned() + new_text + &self.text[range.end..]).into();
        if !self.is_valid_input(&pending_text) {
            return;
        }

        self.push_history(&range, new_text, cx);
        self.text = pending_text;
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.emit(InputEvent::Change(self.text.clone()));
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        cx: &mut ViewContext<Self>,
    ) {
        if self.disabled {
            return;
        }

        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let pending_text: SharedString =
            (self.text[0..range.start].to_owned() + new_text + &self.text[range.end..]).into();
        if !self.is_valid_input(&pending_text) {
            return;
        }

        self.push_history(&range, new_text, cx);
        self.text = pending_text;
        self.marked_range = Some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        cx.emit(InputEvent::Change(self.text.clone()));
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        cx: &mut ViewContext<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line_height = cx.line_height();
        let lines = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);

        let mut start_origin = None;
        let mut end_origin = None;
        for line in lines {
            if let Some(p) = line.position_for_index(range.start, line_height) {
                start_origin = Some(p);
            }
            if let Some(p) = line.position_for_index(range.end, line_height) {
                end_origin = Some(p);
            }

            if start_origin.is_some() && end_origin.is_some() {
                break;
            }
        }

        Some(Bounds::from_corners(
            bounds.origin + start_origin.unwrap_or_default(),
            bounds.origin + end_origin.unwrap_or_default(),
        ))
    }
}

impl FocusableView for TextInput {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        const LINE_HEIGHT: Rems = Rems(1.25);
        let focused = self.focus_handle.is_focused(cx);

        let prefix = self.prefix.as_ref().map(|build| build(cx));
        let suffix = self.suffix.as_ref().map(|build| build(cx));

        div()
            .flex()
            .key_context(CONTEXT)
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
            .on_action(cx.listener(Self::select_to_home))
            .on_action(cx.listener(Self::select_to_end))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::redo))
            // Double click to select all
            .on_double_click(cx.listener(|view, _, cx| {
                view.select_all(&SelectAll, cx);
            }))
            .on_key_down(cx.listener(Self::on_key_down_for_blink_cursor))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            // .on_mouse_move(cx.listener(Self::on_mouse_move))
            .size_full()
            .line_height(LINE_HEIGHT)
            // .text_size(rems(0.875))
            .input_py(self.size)
            .input_h(self.size)
            .when(self.multi_line, |this| this.h_auto())
            .when(self.appearance, |this| {
                this.bg(if self.disabled {
                    cx.theme().muted
                } else {
                    cx.theme().background
                })
                .border_color(cx.theme().input)
                .border_1()
                .rounded(px(cx.theme().radius))
                .when(cx.theme().shadow, |this| this.shadow_sm())
                .when(focused, |this| this.outline(cx))
                .when(prefix.is_none(), |this| this.input_pl(self.size))
                .when(suffix.is_none(), |this| this.input_pr(self.size))
            })
            .children(prefix)
            .gap_1()
            .items_center()
            .child(
                div()
                    .id("TextElement")
                    .flex_grow()
                    .overflow_x_hidden()
                    .cursor_text()
                    .child(TextElement::new(cx.view().clone())),
            )
            .when(self.loading, |this| {
                this.child(Indicator::new().color(cx.theme().muted_foreground))
            })
            .when(
                self.cleanable && !self.loading && !self.text.is_empty(),
                |this| this.child(ClearButton::new(cx).on_click(cx.listener(Self::clean))),
            )
            .children(suffix)
    }
}
