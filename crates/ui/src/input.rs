use std::{f32::INFINITY, ops::Range};

use gpui::{
    div, px, AppContext, ClipboardItem, Div, Element, ElementId, EventEmitter, FocusHandle,
    FocusableElement as _, FocusableView, HighlightStyle, Hsla, InteractiveElement as _,
    InteractiveText, IntoElement, KeyDownEvent, MouseDownEvent, ParentElement as _, Point, Render,
    RenderOnce, Style, Styled, StyledText, TextStyle, View, ViewContext, VisualContext,
    WindowContext,
};
use log::debug;

use crate::{
    cursor::{CursorLayout, CursorShape},
    Color,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InputMode {
    SingleLine,
    AutoHeight { max_lines: usize },
    Full,
}

#[derive(Clone, Debug)]
pub enum SoftWrap {
    None,
    PreferLine,
    EditorWidth,
    Column(u32),
}

pub struct TextView {
    id: ElementId,
    pub text: String,
    pub selection: Range<usize>,
    pub word_click: (usize, u16),
    pub placeholder: String,
    masked: bool,
}

impl EventEmitter<InputEvent> for TextView {}

impl TextView {
    pub fn init(
        id: impl Into<ElementId>,
        cx: &mut WindowContext,
        focus_handle: &FocusHandle,
    ) -> View<Self> {
        let view = Self {
            id: id.into(),
            text: String::new(),
            selection: 0..0,
            word_click: (0, 0),
            placeholder: String::new(),
            masked: false,
        };

        let view = cx.new_view(|cx| {
            #[cfg(debug_assertions)]
            cx.on_release(|_, _, _| debug!("Text Input released"))
                .detach();

            cx.on_blur(focus_handle, |_: &mut TextView, cx| {
                cx.emit(InputEvent::Blur);
            })
            .detach();

            cx.on_focus(focus_handle, |view, cx| {
                view.select_all(cx);
            })
            .detach();

            view
        });
        cx.subscribe(&view, |subscriber, emitter: &InputEvent, cx| {
            if let InputEvent::Input { text: _ } = emitter {
                subscriber.update(cx, |editor, _cx| {
                    editor.word_click = (0, 0);
                });
            }
        })
        .detach();

        view
    }

    pub fn select_all(&mut self, cx: &mut ViewContext<Self>) {
        let len = self.text.chars().count();
        self.selection = 0..len;
        cx.notify()
    }

    pub fn word_ranges(&self) -> Vec<Range<usize>> {
        let mut words = Vec::new();
        let mut last_was_boundary = true;
        let mut word_start = 0;
        let s = self.text.clone();

        for (i, c) in s.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                if last_was_boundary {
                    word_start = i;
                }
                last_was_boundary = false;
            } else {
                if !last_was_boundary {
                    words.push(word_start..i);
                }
                last_was_boundary = true;
            }
        }

        // Check if the last characters form a word and push it if so
        if !last_was_boundary {
            words.push(word_start..s.len());
        }

        words
    }

    pub fn reset(&mut self, cx: &mut ViewContext<Self>) {
        self.text.clear();
        self.selection = 0..0;
        cx.notify();
        cx.emit(InputEvent::Input {
            text: self.text.clone(),
        });
    }

    pub fn set_placeholder(&mut self, placeholder: String, cx: &mut ViewContext<Self>) {
        self.placeholder = placeholder.to_string();
        cx.notify();
    }

    pub fn set_text(&mut self, text: String, cx: &mut ViewContext<Self>) {
        self.text = text;
        let len = self.text.chars().count();
        self.selection = len..len;
        cx.notify();
        cx.emit(InputEvent::Input {
            text: self.text.clone(),
        });
    }

    pub fn char_range_to_text_range(&self, text: &str) -> Range<usize> {
        let start = text
            .chars()
            .take(self.selection.start)
            .collect::<String>()
            .len();
        let end = text
            .chars()
            .take(self.selection.end)
            .collect::<String>()
            .len();
        start..end
    }

    fn cursor_layout(&self, color: Hsla) -> CursorLayout {
        CursorLayout::new(
            Point {
                x: px(0.),
                y: px(0.),
            },
            px(0.0),
            px(20.0),
            color,
            CursorShape::Bar,
            None,
        )
    }
}

impl Render for TextView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let selection_style = HighlightStyle {
            fade_out: Some(0.6),
            background_color: Some(Color::Selection.color(cx)),
            ..Default::default()
        };
        let mut highlights = vec![(self.char_range_to_text_range(&self.text), selection_style)];

        let (display_text, _is_empty, style) = match self.text.is_empty() {
            true => {
                let style = TextStyle {
                    color: Color::Input.color(cx),
                    ..TextStyle::default()
                };

                highlights = vec![];

                (self.placeholder.clone(), true, style)
            }
            false => {
                let style = TextStyle {
                    color: Color::Foreground.color(cx),
                    ..TextStyle::default()
                };

                (self.text.clone(), false, style)
            }
        };

        let styled_text = StyledText::new(display_text + " ").with_highlights(&style, highlights);
        let view = cx.view().clone();

        InteractiveText::new(self.id.clone(), styled_text).on_click(
            self.word_ranges(),
            move |ev, cx| {
                view.update(cx, |editor, cx| {
                    let (index, mut count) = editor.word_click;
                    if index == ev {
                        count += 1;
                    } else {
                        count = 1;
                    }
                    match count {
                        2 => {
                            let word_ranges = editor.word_ranges();
                            editor.selection = word_ranges.get(ev).unwrap().clone();
                        }
                        3 => {
                            // Should select the line
                            let line_start = editor.text[..ev].rfind('\n').map_or(0, |i| i + 1);
                            let line_end = editor.text[ev..]
                                .find('\n')
                                .map_or(editor.text.len(), |i| ev + i);
                            editor.selection = line_start..line_end;
                        }
                        4 => {
                            count = 0;
                            editor.selection = 0..editor.text.len();
                        }
                        _ => {
                            editor.selection = 0..0;
                        }
                    }
                    editor.word_click = (ev, count);
                    cx.notify();
                });
            },
        )
    }
}

pub struct Input {
    id: ElementId,
    base: Div,
    focus_handle: FocusHandle,
    mode: InputMode,
    wrap: SoftWrap,
    view: View<TextView>,
}

impl Styled for Input {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl Input {
    pub fn new(id: impl Into<ElementId>, cx: &mut WindowContext) -> Self {
        let focus_handle = cx.focus_handle();
        let id = id.into();
        let input = Self {
            id: id.clone(),
            base: div(),
            view: TextView::init(id, cx, &focus_handle),
            focus_handle,
            wrap: SoftWrap::None,
            mode: InputMode::SingleLine,
        };

        input
    }

    pub fn wrap(mut self, wrap: SoftWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set_placeholder<C: VisualContext>(&self, placeholder: impl ToString, cx: &mut C) {
        cx.update_view(&self.view, |editor: &mut TextView, cx| {
            editor.set_placeholder(placeholder.to_string(), cx);
        });
    }

    pub fn set_text<C: VisualContext>(&self, text: impl ToString, cx: &mut C) {
        cx.update_view(&self.view, |editor: &mut TextView, cx| {
            editor.set_text(text.to_string(), cx);
        });
    }

    fn handle_focus(&mut self, cx: &mut ViewContext<Self>) {
        cx.emit(InputEvent::Focus);
    }

    fn handle_key_down(&mut self, ev: &KeyDownEvent, cx: &mut ViewContext<Self>) {
        cx.emit(InputEvent::KeyDown(ev.clone()));
        let keystroke = &ev.keystroke.key;

        #[cfg(target_os = "macos")]
        let modifier = ev.keystroke.modifiers.platform;
        #[cfg(not(target_os = "macos"))]
        let modifier = ev.keystroke.modifiers.control;

        self.view.update(cx, |editor, cx| {
            let prev = editor.text.clone();
            let chars = editor.text.chars().collect::<Vec<char>>();

            #[cfg(not(target_os = "macos"))]
            if ev.keystroke.modifiers.control {
                if self.mode == InputMode::SingleLine {
                    match keystroke.as_str() {
                        "a" => {
                            // Move cursor to the beginning of the line
                            editor.selection = 0..0;
                        }
                        "e" => {
                            // Move cursor to the end of the line
                            editor.selection = chars.len()..chars.len();
                        }
                        _ => {}
                    }
                }
            }

            if modifier {
                match keystroke.as_str() {
                    "a" => {
                        editor.selection = 0..chars.len();
                    }
                    "c" => {
                        if !editor.masked {
                            let selected_text = chars[editor.selection.clone()].iter().collect();
                            cx.write_to_clipboard(ClipboardItem::new(selected_text));
                        }
                    }
                    "v" => {
                        let clipboard = cx.read_from_clipboard();
                        if let Some(clipboard) = clipboard {
                            let text = clipboard.text();
                            editor
                                .text
                                .replace_range(editor.char_range_to_text_range(&editor.text), text);
                            let i = editor.selection.start + text.chars().count();
                            editor.selection = i..i;
                        }
                    }
                    "x" => {
                        let selected_text = chars[editor.selection.clone()].iter().collect();
                        cx.write_to_clipboard(ClipboardItem::new(selected_text));
                        editor
                            .text
                            .replace_range(editor.char_range_to_text_range(&editor.text), "");
                        editor.selection.end = editor.selection.start;
                    }
                    _ => {}
                }
            } else if !ev.keystroke.ime_key.clone().unwrap_or_default().is_empty() {
                let ime_key = &ev.keystroke.ime_key.clone().unwrap_or_default();
                editor
                    .text
                    .replace_range(editor.char_range_to_text_range(&editor.text), ime_key);
                let i = editor.selection.start + ime_key.chars().count();
                editor.selection = i..i;
            } else {
                match keystroke.as_str() {
                    "backspace" => {
                        if editor.text.is_empty() && !ev.is_held {
                            cx.emit(InputEvent::Back);
                        } else if editor.selection.start == editor.selection.end
                            && editor.selection.start > 0
                        {
                            let i = (editor.selection.start - 1).min(chars.len());
                            editor.text = chars[0..i].iter().collect::<String>()
                                + &(chars[editor.selection.end.min(chars.len())..]
                                    .iter()
                                    .collect::<String>());
                            editor.selection = i..i;
                        } else {
                            editor
                                .text
                                .replace_range(editor.char_range_to_text_range(&editor.text), "");
                            editor.selection.end = editor.selection.start;
                        }
                    }
                    "delete" => {
                        if editor.text.is_empty() && !ev.is_held {
                            cx.emit(InputEvent::Back);
                        } else if editor.selection.start == editor.selection.end
                            && editor.selection.end < editor.text.len()
                        {
                            let i = editor.selection.start.min(chars.len());
                            editor.text = chars[0..i].iter().collect::<String>()
                                + &(chars[editor.selection.end.min(chars.len())..]
                                    .iter()
                                    .collect::<String>());
                            editor.selection = i..i;
                        } else {
                            editor
                                .text
                                .replace_range(editor.char_range_to_text_range(&editor.text), "");
                            editor.selection.end = editor.selection.start;
                        }
                    }
                    "left" => {
                        if editor.selection.start > 0 {
                            let i = if editor.selection.start == editor.selection.end {
                                editor.selection.start - 1
                            } else {
                                editor.selection.start
                            };
                            editor.selection = i..i;
                        }
                    }
                    "right" => {
                        if editor.selection.end < editor.text.len() {
                            let i = if editor.selection.start == editor.selection.end {
                                editor.selection.end + 1
                            } else {
                                editor.selection.end
                            };
                            editor.selection = i..i;
                        }
                    }
                    "up" => {}
                    "down" => {}
                    "enter" => {
                        if self.mode == InputMode::SingleLine {
                            cx.emit(InputEvent::Blur);
                        } else {
                            editor.text.insert(editor.selection.start, '\n');
                            editor.selection.start += 1;
                            editor.selection.end += 1;
                        }
                    }
                    "tab" => {
                        if self.mode == InputMode::SingleLine {
                            cx.emit(InputEvent::Blur);
                        } else {
                            editor.text.insert(editor.selection.start, '\t');
                            editor.selection.start += 1;
                            editor.selection.end += 1;
                        }
                    }
                    _ => {}
                }
            }

            debug!(
                "------------------- {} {:?}",
                &editor.text, &editor.selection
            );
            if prev != editor.text {
                cx.emit(InputEvent::Input {
                    text: editor.text.clone(),
                });
            }
            cx.notify();
        })
    }
}

pub enum InputEvent {
    Input { text: String },
    Focus,
    Blur,
    Back,
    KeyDown(KeyDownEvent),
}

impl EventEmitter<InputEvent> for Input {}

impl FocusableView for Input {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Input {
    fn render<'a>(&mut self, cx: &mut ViewContext<'a, Self>) -> impl IntoElement {
        let text_view = self.view.clone();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_any_mouse_down(cx.listener(|this, _, cx| {
                debug!("on_any_mouse_down");
                cx.focus(&this.focus_handle);
            }))
            .cursor_text()
            .border_1()
            .border_color(Color::Input.color(cx))
            .bg(Color::Background.color(cx))
            .focus(|style| style.border_color(Color::Ring.color(cx)))
            .py_1()
            .px_3()
            .rounded_md()
            .child(text_view)
    }
}

pub struct InputLayout {}

// impl Element for Input {
//     type RequestLayoutState = ();

//     type PrepaintState = InputLayout;

//     fn id(&self) -> Option<gpui::ElementId> {
//         None
//     }

//     fn request_layout(
//         &mut self,
//         id: Option<&gpui::GlobalElementId>,
//         cx: &mut WindowContext,
//     ) -> (gpui::LayoutId, Self::RequestLayoutState) {
//         let style = Style::default();
//         let layout_id = cx.request_layout(style, None);
//         (layout_id, ())
//     }

//     fn prepaint(
//         &mut self,
//         id: Option<&gpui::GlobalElementId>,
//         bounds: gpui::Bounds<gpui::Pixels>,
//         request_layout: &mut Self::RequestLayoutState,
//         cx: &mut WindowContext,
//     ) -> Self::PrepaintState {
//         InputLayout {}
//     }

//     fn paint(
//         &mut self,
//         id: Option<&gpui::GlobalElementId>,
//         bounds: gpui::Bounds<gpui::Pixels>,
//         request_layout: &mut Self::RequestLayoutState,
//         prepaint: &mut Self::PrepaintState,
//         cx: &mut WindowContext,
//     ) {
//     }
// }
