mod blink_manager;
mod cursor_layout;
mod text_view;

use crate::{h_flex, theme::ActiveTheme};
use gpui::{
    div, prelude::FluentBuilder as _, AppContext, ClipboardItem, Div, EventEmitter, FocusHandle,
    FocusableView, InteractiveElement, Interactivity, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Render, RenderOnce, SharedString, Style, StyleRefinement, Styled, View,
    ViewContext, WindowContext,
};
use std::{sync::Arc, time::Duration};
use text_view::TextView;

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub struct TextField {
    focus_handle: FocusHandle,
    appearance: bool,
    pub view: View<TextView>,
}

impl TextField {
    pub fn new(cx: &mut WindowContext) -> Self {
        let focus_handle = cx.focus_handle();
        let view = TextView::init(cx, &focus_handle);

        Self {
            focus_handle,
            view,
            appearance: true,
        }
    }

    pub fn focus(&mut self, cx: &mut WindowContext) {
        cx.focus(&self.focus_handle);
    }

    /// Set the appearance of the text field.
    ///
    /// If false, the text field will not have a border, background and focus ring.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut WindowContext,
    ) {
        self.view.update(cx, |text_view, cx| {
            text_view.set_placeholder(placeholder, cx)
        });
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut WindowContext) {
        self.view
            .update(cx, |text_view, cx| text_view.set_disabled(disabled, cx));
    }

    pub fn set_text(&mut self, text: &str, cx: &mut WindowContext) {
        self.view
            .update(cx, |text_view, cx| text_view.set_text(text, cx));
    }

    pub fn text(&self, cx: &AppContext) -> String {
        self.view.read(cx).text.clone()
    }

    pub fn set_masked(&mut self, masked: bool, cx: &mut WindowContext) {
        self.view
            .update(cx, |text_view, cx| text_view.set_masked(masked, cx));
    }
}

impl FocusableView for TextField {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextField {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let theme = cx.theme();

        let view = self.view.clone();
        let text_view = view.read(cx);

        let focused = self.focus_handle.is_focused(cx);
        let disabled = text_view.disabled;

        h_flex()
            .w_full()
            .track_focus(&focus_handle)
            .when(!text_view.disabled, |this| {
                this.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |view, _, cx| {
                        cx.prevent_default();
                        view.focus_handle.focus(cx)
                    }),
                )
            })
            .when(!disabled, |this| {
                this.on_key_down(cx.listener(move |this, ev: &KeyDownEvent, cx| {
                    this.view.update(cx, |text_view, cx| {
                        let prev = text_view.text.clone();
                        cx.emit(TextEvent::KeyDown(ev.clone()));
                        let keystroke = ev.keystroke.key.as_str();
                        let chars = text_view.text.chars().collect::<Vec<char>>();
                        let m = ev.keystroke.modifiers.secondary();

                        if m {
                            match keystroke {
                                "a" => {
                                    text_view.selection = 0..chars.len();
                                }
                                "c" => {
                                    // if !text_view.masked {
                                    let selected_text =
                                        chars[text_view.selection.clone()].iter().collect();
                                    cx.write_to_clipboard(ClipboardItem::new(selected_text));
                                    // }
                                }
                                "v" => {
                                    let clipboard = cx.read_from_clipboard();
                                    if let Some(clipboard) = clipboard {
                                        let text = clipboard.text();
                                        text_view.text.replace_range(
                                            text_view.char_range_to_text_range(&text_view.text),
                                            text,
                                        );
                                        let i = text_view.selection.start + text.chars().count();
                                        text_view.selection = i..i;
                                    }
                                }
                                "x" => {
                                    let selected_text =
                                        chars[text_view.selection.clone()].iter().collect();
                                    cx.write_to_clipboard(ClipboardItem::new(selected_text));
                                    text_view.text.replace_range(
                                        text_view.char_range_to_text_range(&text_view.text),
                                        "",
                                    );
                                    text_view.selection.end = text_view.selection.start;
                                }
                                _ => {}
                            }
                        } else if ev.keystroke.modifiers.control {
                            // On macOS, ctrl+a, ctrl+e are used for moving cursor to start/end of line
                            match keystroke {
                                "a" => {
                                    // Move cursor to first of line
                                    text_view.selection = 0..0;
                                }
                                "e" => {
                                    // Move cursor to end of line
                                    text_view.selection = chars.len()..chars.len();
                                }
                                _ => {}
                            }
                        } else if !ev.keystroke.ime_key.clone().unwrap_or_default().is_empty() {
                            let ime_key = &ev.keystroke.ime_key.clone().unwrap_or_default();
                            text_view.text.replace_range(
                                text_view.char_range_to_text_range(&text_view.text),
                                ime_key,
                            );
                            let i = text_view.selection.start + ime_key.chars().count();
                            text_view.selection = i..i;
                        } else {
                            match keystroke {
                                "left" => {
                                    if text_view.selection.start > 0 {
                                        let i = if text_view.selection.start
                                            == text_view.selection.end
                                        {
                                            text_view.selection.start - 1
                                        } else {
                                            text_view.selection.start
                                        };
                                        text_view.selection = i..i;
                                    }
                                }
                                "right" => {
                                    if text_view.selection.end < text_view.text.len() {
                                        let i = if text_view.selection.start
                                            == text_view.selection.end
                                        {
                                            text_view.selection.end + 1
                                        } else {
                                            text_view.selection.end
                                        };
                                        text_view.selection = i..i;
                                    }
                                }
                                "backspace" => {
                                    if text_view.text.is_empty() && !ev.is_held {
                                        // cx.emit(TextEvent::Back);
                                    } else if text_view.selection.start == text_view.selection.end
                                        && text_view.selection.start > 0
                                    {
                                        let i = (text_view.selection.start - 1).min(chars.len());
                                        text_view.text = chars[0..i].iter().collect::<String>()
                                            + &(chars[text_view.selection.end.min(chars.len())..]
                                                .iter()
                                                .collect::<String>());
                                        text_view.selection = i..i;
                                    } else {
                                        text_view.text.replace_range(
                                            text_view.char_range_to_text_range(&text_view.text),
                                            "",
                                        );
                                        text_view.selection.end = text_view.selection.start;
                                    }
                                }
                                "enter" => {
                                    if ev.keystroke.modifiers.shift {
                                        text_view.text.insert(
                                            text_view
                                                .char_range_to_text_range(&text_view.text)
                                                .start,
                                            '\n',
                                        );
                                        let i = text_view.selection.start + 1;
                                        text_view.selection = i..i;
                                    }
                                }
                                _ => {}
                            };
                        }

                        if prev != text_view.text {
                            cx.emit(TextEvent::Input {
                                text: text_view.text.clone(),
                            });
                        }
                        cx.notify();
                    });
                }))
            })
            .when(self.appearance, |this| {
                this.border_color(if focused { theme.ring } else { theme.input })
                    .border_1()
                    .rounded_sm()
                    .py_1()
                    .px_3()
                    .h_9()
                    .shadow_sm()
                    .bg(if disabled {
                        theme.muted
                    } else {
                        theme.background
                    })
            })
            .min_w_20()
            .child(view)
    }
}

pub enum TextEvent {
    Input { text: String },
    Blur,
    Focus,
    KeyDown(KeyDownEvent),
}

impl EventEmitter<TextEvent> for TextField {}
