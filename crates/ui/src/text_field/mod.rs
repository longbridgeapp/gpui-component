mod blink_manager;
mod cursor_layout;
mod text_view;

use crate::{disableable::Disableable, theme::Theme};
use blink_manager::BlinkManager;
use gpui::{
    div, px, relative, ClipboardItem, Context, Element, EventEmitter, FocusHandle, HighlightStyle,
    InteractiveElement, InteractiveText, IntoElement, KeyDownEvent, Model, ParentElement, Pixels,
    Render, Style, Styled, StyledText, TextStyle, View, ViewContext, VisualContext, WindowContext,
};
use std::{ops::Range, time::Duration};
use text_view::TextView;

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone)]
pub struct TextField {
    focus_handle: FocusHandle,
    disable: bool,
    blink_manager: Model<BlinkManager>,
    pub view: View<TextView>,
}

impl TextField {
    pub fn new(placeholder: &str, disable: bool, cx: &mut ViewContext<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let view = TextView::init(cx, &focus_handle, placeholder, disable);

        let blink_manager = cx.new_model(|cx| BlinkManager::new(CURSOR_BLINK_INTERVAL, cx));

        cx.on_focus(&focus_handle, Self::handle_focus).detach();
        cx.on_blur(&focus_handle, Self::handle_blur).detach();

        Self {
            focus_handle,
            view,
            blink_manager,
            disable,
        }
    }

    pub fn focus(&self, cx: &mut WindowContext) {
        cx.focus(&self.focus_handle);
    }

    fn handle_focus(&mut self, cx: &mut ViewContext<Self>) {
        cx.emit(TextEvent::Focus);
        self.blink_manager.update(cx, BlinkManager::enable);
        cx.notify();
    }

    fn handle_blur(&mut self, cx: &mut ViewContext<Self>) {
        cx.emit(TextEvent::Blur);
        self.blink_manager.update(cx, BlinkManager::disable);
        cx.notify();
    }

    pub fn show_cursor(&self, cx: &mut WindowContext) -> bool {
        self.blink_manager.read(cx).visible() && self.focus_handle.is_focused(cx)
    }
}

impl Disableable for TextField {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disable = disabled;
        self
    }
}

impl Render for TextField {
    fn render<'a>(&mut self, cx: &mut ViewContext<'a, Self>) -> impl IntoElement {
        cx.focus(&self.focus_handle);
        let theme = cx.global::<Theme>();

        let text_view = self.view.clone();
        let text_view1 = text_view.clone();

        div()
            .border_color(if self.focus_handle.is_focused(cx) {
                theme.blue
            } else {
                theme.crust
            })
            .border_1()
            .track_focus(&self.focus_handle)
            .on_key_down(move |ev, cx| {
                text_view1.update(cx, |editor, cx| {
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
            .child(text_view)
    }
}

pub enum TextEvent {
    Input { text: String },
    Blur,
    Focus,
    KeyDown(KeyDownEvent),
}

impl EventEmitter<TextEvent> for TextField {}
