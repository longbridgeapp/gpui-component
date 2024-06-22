use std::{ops::Range, time::Duration};

use gpui::{
    div, ClipboardItem, Context, EventEmitter, FocusHandle, HighlightStyle, InteractiveElement,
    InteractiveText, IntoElement, KeyDownEvent, Model, ParentElement, Render, RenderOnce, Styled,
    StyledText, TextStyle, View, ViewContext, VisualContext, WindowContext,
};

use crate::{disableable::Disableable, theme::Theme};

use super::{blink_manager::BlinkManager, cursor_layout::CursorLayout};

#[derive(IntoElement, Clone)]
pub struct TextField {
    focus_handle: FocusHandle,
    disable: bool,
    pub view: View<TextView>,
}

impl TextField {
    pub fn new(cx: &mut WindowContext, placeholder: &str, disable: bool) -> Self {
        let focus_handle = cx.focus_handle();
        let view = TextView::init(cx, &focus_handle, placeholder, disable);

        Self {
            focus_handle,
            view,
            disable,
        }
    }

    pub fn focus(&self, cx: &mut WindowContext) {
        cx.focus(&self.focus_handle);
    }
}

impl Disableable for TextField {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disable = disabled;
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        cx.focus(&self.focus_handle);
        let theme = cx.global::<Theme>();

        let clone = self.view.clone();

        div()
            .border_color(if self.focus_handle.is_focused(cx) {
                theme.blue
            } else {
                theme.crust
            })
            .border_1()
            .track_focus(&self.focus_handle)
            .on_key_down(move |ev, cx| {
                self.view.update(cx, |editor, cx| {
                    let prev = editor.text.clone();
                    cx.emit(TextEvent::KeyDown(ev.clone()));
                    let keystroke = &ev.keystroke.key;
                    let chars = editor.text.chars().collect::<Vec<char>>();
                    let m = ev.keystroke.modifiers.secondary();

                    dbg!("---------------- {:?}", ev);

                    if m {
                        match keystroke.as_str() {
                            "a" => {
                                editor.selection = 0..chars.len();
                            }
                            "c" => {
                                // if !editor.masked {
                                let selected_text =
                                    chars[editor.selection.clone()].iter().collect();
                                cx.write_to_clipboard(ClipboardItem::new(selected_text));
                                // }
                            }
                            "v" => {
                                let clipboard = cx.read_from_clipboard();
                                if let Some(clipboard) = clipboard {
                                    let text = clipboard.text();
                                    editor.text.replace_range(
                                        editor.char_range_to_text_range(&editor.text),
                                        text,
                                    );
                                    let i = editor.selection.start + text.chars().count();
                                    editor.selection = i..i;
                                }
                            }
                            "x" => {
                                let selected_text =
                                    chars[editor.selection.clone()].iter().collect();
                                cx.write_to_clipboard(ClipboardItem::new(selected_text));
                                editor.text.replace_range(
                                    editor.char_range_to_text_range(&editor.text),
                                    "",
                                );
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
                            "backspace" => {
                                if editor.text.is_empty() && !ev.is_held {
                                    // cx.emit(TextEvent::Back);
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
                                    editor.text.replace_range(
                                        editor.char_range_to_text_range(&editor.text),
                                        "",
                                    );
                                    editor.selection.end = editor.selection.start;
                                }
                            }
                            "enter" => {
                                if ev.keystroke.modifiers.shift {
                                    editor.text.insert(
                                        editor.char_range_to_text_range(&editor.text).start,
                                        '\n',
                                    );
                                    let i = editor.selection.start + 1;
                                    editor.selection = i..i;
                                }
                            }
                            _ => {}
                        };
                    }
                    if prev != editor.text {
                        cx.emit(TextEvent::Input {
                            text: editor.text.clone(),
                        });
                    }
                    cx.notify();
                });
            })
            .rounded_sm()
            .py_1p5()
            .px_3()
            .min_w_20()
            .bg(if self.disable {
                theme.crust
            } else {
                theme.base
            })
            .child(clone)
    }
}

pub enum TextEvent {
    Input { text: String },
    Blur,
    KeyDown(KeyDownEvent),
}

impl EventEmitter<TextEvent> for TextView {}

pub struct TextView {
    pub text: String,
    pub placeholder: String,
    pub word_click: (usize, u16),
    pub selection: Range<usize>,
    pub disable: bool,
    pub blink_manager: Model<BlinkManager>,
    pub cursor: CursorLayout,
}

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

impl TextView {
    pub fn init(
        cx: &mut WindowContext,
        focus_handle: &FocusHandle,
        placeholder: &str,
        disable: bool,
    ) -> View<Self> {
        let blink_manager = cx.new_model(|_cx| BlinkManager::new(CURSOR_BLINK_INTERVAL));

        let cursor = CursorLayout::new(
            gpui::Point::new(gpui::px(0.0), gpui::px(0.0)),
            gpui::px(2.0),
            gpui::px(20.0),
            gpui::Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 1.0,
            },
            None,
        );

        let m = Self {
            text: String::new(),
            placeholder: placeholder.to_string(),
            word_click: (0, 0),
            selection: 0..0,
            blink_manager,
            cursor,
            disable,
        };

        let view = cx.new_view(|cx| {
            cx.on_blur(
                focus_handle,
                |view: &mut TextView, cx: &mut ViewContext<'_, TextView>| {
                    view.blink_manager.update(cx, BlinkManager::disable);
                    cx.emit(TextEvent::Blur);
                },
            )
            .detach();

            cx.on_focus(focus_handle, |view, cx| {
                view.blink_manager.update(cx, |bm, cx| {
                    bm.blink_cursor(0, cx);
                });
            })
            .detach();
            m
        });

        cx.subscribe(
            &view,
            move |subscriber, emitter: &TextEvent, cx| match emitter {
                TextEvent::Input { text: _ } => {
                    subscriber.update(cx, |editor, cx| {
                        editor.word_click = (0, 0);
                    });
                }
                TextEvent::Blur => {
                    subscriber.update(cx, |editor, cx| {
                        editor.blink_manager.update(cx, BlinkManager::disable);
                        editor.word_click = (0, 0);
                    });
                }
                _ => {}
            },
        )
        .detach();

        view
    }

    pub fn select_all(&mut self, cx: &mut ViewContext<Self>) {
        self.selection = 0..self.text.len();
        cx.notify();
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

    pub fn set_text(&mut self, text: impl ToString, cx: &mut ViewContext<Self>) {
        self.text = text.to_string();
        self.selection = self.text.len()..self.text.len();
        cx.notify();
        cx.emit(TextEvent::Input {
            text: self.text.clone(),
        });
    }
}

impl Render for TextView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let mut text = self.text.clone();

        let mut style = TextStyle {
            color: theme.text,
            font_family: theme.font_sans.clone(),
            ..Default::default()
        };

        let mut selection_style = HighlightStyle::default();
        let mut color = theme.lavender;
        color.fade_out(0.8);
        selection_style.background_color = Some(color);

        let highlights = vec![(self.char_range_to_text_range(&text), selection_style)];

        let styled_text: StyledText = if text.is_empty() {
            text = self.placeholder.to_string();
            style.color = theme.subtext0;
            StyledText::new(text).with_highlights(&style, highlights)
        } else {
            StyledText::new(text).with_highlights(&style, highlights)
        };

        let view = cx.view().clone();

        InteractiveText::new("text", styled_text).on_click(self.word_ranges(), move |ev, cx| {
            view.update(cx, |text_view, cx| {
                let (index, mut count) = text_view.word_click;
                if index == ev {
                    count += 1;
                } else {
                    count = 1;
                }
                match count {
                    2 => {
                        let word_ranges = text_view.word_ranges();
                        text_view.selection = word_ranges.get(ev).unwrap().clone();
                    }
                    3 => {
                        // Should select the line
                    }
                    4 => {
                        count = 0;
                        text_view.selection = 0..text_view.text.len();
                    }
                    _ => {}
                }
                text_view.word_click = (ev, count);
                cx.notify();
            });
        })
    }
}
