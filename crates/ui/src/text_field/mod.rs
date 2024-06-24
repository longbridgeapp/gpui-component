mod blink_manager;
mod cursor_layout;
mod text_view;

use crate::{disableable::Disableable, theme::Theme};
use blink_manager::BlinkManager;
use gpui::{
    div, ClipboardItem, Context, EventEmitter, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, Model, ParentElement, Render, RenderOnce, Styled, View, ViewContext,
    WindowContext,
};
use std::time::Duration;
use text_view::TextView;

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, IntoElement)]
pub struct TextField {
    focus_handle: FocusHandle,
    disable: bool,
    blink_manager: Model<BlinkManager>,
    pub view: View<TextView>,
}

impl TextField {
    pub fn new(placeholder: &str, disable: bool, cx: &mut WindowContext) -> Self {
        let focus_handle = cx.focus_handle();
        let view = TextView::init(cx, &focus_handle, placeholder, disable);

        let blink_manager = cx.new_model(|cx| BlinkManager::new(CURSOR_BLINK_INTERVAL, cx));

        // cx.on_focus(&focus_handle, Self::handle_focus).detach();
        // cx.on_blur(&focus_handle, Self::handle_blur).detach();

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
impl RenderOnce for TextField {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        cx.focus(&self.focus_handle);
        let theme = cx.global::<Theme>();

        let view = self.view.clone();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(move |ev, cx| {
                self.view.update(cx, |text_view, cx| {
                    let prev = text_view.text.clone();
                    cx.emit(TextEvent::KeyDown(ev.clone()));
                    let keystroke = &ev.keystroke.key;
                    let chars = text_view.text.chars().collect::<Vec<char>>();
                    let m = ev.keystroke.modifiers.secondary();

                    dbg!(&text_view.text, &text_view.selection);

                    if m {
                        match keystroke.as_str() {
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
                    } else if !ev.keystroke.ime_key.clone().unwrap_or_default().is_empty() {
                        let ime_key = &ev.keystroke.ime_key.clone().unwrap_or_default();
                        text_view.text.replace_range(
                            text_view.char_range_to_text_range(&text_view.text),
                            ime_key,
                        );
                        let i = text_view.selection.start + ime_key.chars().count();
                        text_view.selection = i..i;
                    } else {
                        match keystroke.as_str() {
                            "left" => {
                                if text_view.selection.start > 0 {
                                    let i = if text_view.selection.start == text_view.selection.end
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
                                    let i = if text_view.selection.start == text_view.selection.end
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
                                        text_view.char_range_to_text_range(&text_view.text).start,
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
                        dbg!(&text_view.text, &text_view.selection);
                    }
                    cx.notify();
                });
            })
            .border_color(if self.focus_handle.is_focused(cx) {
                theme.blue
            } else {
                theme.crust
            })
            .border_1()
            .rounded_sm()
            .py_1p5()
            .px_3()
            .min_w_20()
            .bg(if self.disable {
                theme.crust
            } else {
                theme.base
            })
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
