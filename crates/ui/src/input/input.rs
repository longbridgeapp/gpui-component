//! A text input field that allows the user to enter text.
//!
//! Based on the `Input` example from the `gpui` crate.
//! https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/input.rs

use std::ops::Range;

use super::blink_cursor::BlinkCursor;
use super::change::Change;
use super::ClearButton;
use crate::history::History;
use crate::indicator::Indicator;
use crate::theme::ActiveTheme;
use crate::StyledExt as _;
use crate::{event::InteractiveElementExt as _, Size};
use crate::{Sizable, StyleSized};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    actions, div, fill, point, px, relative, rems, size, AnyElement, AppContext, Bounds,
    ClickEvent, ClipboardItem, Context as _, Element, ElementId, ElementInputHandler, EventEmitter,
    FocusHandle, FocusableView, GlobalElementId, InteractiveElement as _, IntoElement, KeyBinding,
    KeyDownEvent, LayoutId, Model, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PaintQuad, ParentElement as _, Pixels, Point, Render, ShapedLine, SharedString, Style,
    Styled as _, TextRun, UnderlineStyle, View, ViewContext, ViewInputHandler, WindowContext,
};
use unicode_segmentation::*;

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
        KeyBinding::new("home", Home, Some(CONTEXT)),
        KeyBinding::new("end", End, Some(CONTEXT)),
        KeyBinding::new("shift-home", SelectToHome, Some(CONTEXT)),
        KeyBinding::new("shift-end", SelectToEnd, Some(CONTEXT)),
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
        KeyBinding::new("ctrl-e", End, Some(CONTEXT)),
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
    focus_handle: FocusHandle,
    text: SharedString,
    history: History<Change>,
    blink_cursor: Model<BlinkCursor>,
    prefix: Option<Box<dyn Fn(&mut ViewContext<Self>) -> AnyElement + 'static>>,
    suffix: Option<Box<dyn Fn(&mut ViewContext<Self>) -> AnyElement + 'static>>,
    loading: bool,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll_offset: Point<Pixels>,
    is_selecting: bool,
    disabled: bool,
    masked: bool,
    appearance: bool,
    cleanable: bool,
    size: Size,
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
            blink_cursor,
            history,
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
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

    /// Set the text of the input field.
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut ViewContext<Self>) {
        self.history.ignore = true;
        let text: SharedString = text.into();
        let range = 0..self.text.chars().map(|c| c.len_utf16()).sum();
        self.replace_text_in_range(Some(range), &text, cx);
        self.history.ignore = false;

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

    ///  Set the placeholder text without ownership.
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
        cx.emit(InputEvent::PressEnter);
    }

    fn clean(&mut self, _: &ClickEvent, cx: &mut ViewContext<Self>) {
        self.set_text("", cx);
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut ViewContext<Self>) {
        self.is_selecting = true;
        let offset = self.index_for_mouse_position(event.position);

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

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut ViewContext<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
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
        self.replace_text_in_range(None, "", cx);
    }

    fn paste(&mut self, _: &Paste, cx: &mut ViewContext<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            let new_text = clipboard.text().replace('\n', "");
            self.replace_text_in_range(None, &new_text, cx);
        }
    }

    fn push_history(&mut self, range: &Range<usize>, new_text: &str, cx: &mut ViewContext<Self>) {
        if self.history.ignore {
            return;
        }

        let old_text = self
            .text_for_range(self.range_to_utf16(&range), cx)
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

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        // If the text is empty, always return 0
        if self.text.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.text.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
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
        let prev_text = self.text_for_range(0..start, cx).unwrap_or_default();
        let next_text = self
            .text_for_range(end..self.text.len(), cx)
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
struct PrepaintState {
    scroll_offset: Point<Pixels>,
    line: Option<ShapedLine>,
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
        let line = cx
            .text_system()
            .shape_line(display_text, font_size, &runs)
            .unwrap();

        // Calculate the scroll offset to keep the cursor in view
        let mut scroll_offset = input.scroll_offset;
        let mut bounds = bounds;
        let right_margin = px(5.);
        let cursor_pos = line.x_for_index(cursor);
        let cursor_start = line.x_for_index(selected_range.start);
        let cursor_end = line.x_for_index(selected_range.end);

        scroll_offset.x = if scroll_offset.x + cursor_pos > (bounds.size.width - right_margin) {
            // cursor is out of right
            bounds.size.width - right_margin - cursor_pos
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

        bounds.origin = bounds.origin + scroll_offset;

        let inset = px(0.5);
        let (selection, cursor) = if selected_range.is_empty() && input.show_cursor(cx) {
            // cursor blink
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top() + inset),
                        size(px(1.5), bounds.bottom() - bounds.top() - inset * 2),
                    ),
                    crate::blue_500(),
                )),
            )
        } else {
            // selection background
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

        PrepaintState {
            scroll_offset,
            bounds,
            line: Some(line),
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
            input.scroll_offset = prepaint.scroll_offset;
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
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
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .size_full()
            .line_height(rems(1.25))
            .text_size(rems(0.875))
            .input_py(self.size)
            .input_h(self.size)
            .when(self.appearance, |this| {
                this.bg(if self.disabled {
                    cx.theme().muted
                } else {
                    cx.theme().background
                })
                .border_color(cx.theme().input)
                .border_1()
                .rounded(px(cx.theme().radius))
                .shadow_sm()
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
                    .child(TextElement {
                        input: cx.view().clone(),
                    }),
            )
            .when(self.loading, |this| this.child(Indicator::new()))
            .when(
                self.cleanable && !self.loading && !self.text.is_empty(),
                |this| this.child(ClearButton::new(cx).on_click(cx.listener(Self::clean))),
            )
            .children(suffix)
    }
}
