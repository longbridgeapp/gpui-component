use std::{ops::Range, time::Duration};

use gpui::{
    div, Context, EventEmitter, FocusHandle, HighlightStyle, InteractiveElement, InteractiveText,
    IntoElement, KeyDownEvent, Model, ParentElement, Render, RenderOnce, Styled, StyledText,
    TextStyle, View, ViewContext, VisualContext, WindowContext,
};

use crate::{theme::Theme, disableable::Disableable};

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
        // cx.focus(&self.focus_handle);
        let theme = cx.global::<Theme>();

        let clone = self.view.clone();

        div()
            .border_color(
                self.focus_handle
                    .is_focused(cx)
                    .then(|| theme.blue)
                    .unwrap_or(theme.crust),
            )
            .border_1()
            .track_focus(&self.focus_handle)
            .on_key_down(move |event, cx| {
                if self.disable {
                    return;
                }

                self.view.update(cx, |text_view, vc| {
                    let prev = text_view.text.clone();
                    vc.emit(TextEvent::KeyDown(event.clone()));
                    let keystroke = &event.keystroke.key;
                    let chars = text_view.text.chars().collect::<Vec<char>>();

                    let m = event.keystroke.modifiers.platform;

                    if m {
                        match keystroke.as_str() {
                            _ => {}
                        }
                    } else if !event
                        .keystroke
                        .ime_key
                        .clone()
                        .unwrap_or_default()
                        .is_empty()
                    {
                        let ime_key = &event.keystroke.ime_key.clone().unwrap_or_default();
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
                                    text_view.selection =
                                        text_view.selection.start - 1..text_view.selection.end;
                                }
                            }
                            "right" => {
                                if text_view.selection.end < text_view.text.len() {
                                    text_view.selection =
                                        text_view.selection.start + 1..text_view.selection.end + 1;
                                } else {
                                    text_view.selection =
                                        text_view.selection.start + 1..text_view.selection.end;
                                }
                            }
                            "backspace" => {
                                if text_view.text.is_empty() {
                                    return;
                                }

                                if text_view.selection.start == text_view.selection.end {
                                    let i = (text_view.selection.start - 1).min(chars.len());
                                    text_view.text = chars[0..i].iter().collect::<String>()
                                        + &(chars[text_view.selection.end.min(chars.len())..]
                                            .iter()
                                            .collect::<String>());
                                    text_view.selection = i..i;
                                }

                                text_view.text.replace_range(
                                    text_view.char_range_to_text_range(&text_view.text),
                                    "",
                                );

                                text_view.selection.end = text_view.selection.start;
                            }
                            _ => {}
                        }
                    }

                    if prev != text_view.text {
                        vc.emit(TextEvent::Input {
                            text: text_view.text.clone(),
                        });
                    }

                    vc.notify();
                })
            })
            .rounded_lg()
            .py_1p5()
            .px_3()
            .min_w_20()
            .bg(self.disable.then(|| theme.crust).unwrap_or(theme.base))
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
        let blink_manager = cx.new_model(|cx| BlinkManager::new(CURSOR_BLINK_INTERVAL));

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
                |editor: &mut TextView, cx: &mut ViewContext<'_, TextView>| {
                    editor.blink_manager.update(cx, BlinkManager::disable);
                    cx.emit(TextEvent::Blur);
                },
            )
            .detach();

            cx.on_focus(focus_handle, |view, cx| {
                view.select_all(cx);
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
}

impl Render for TextView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let mut text = self.text.clone();

        let mut style = TextStyle::default();
        style.color = theme.text;
        style.font_family = theme.font_sans.clone();

        let mut selection_style = HighlightStyle::default();
        let mut color = theme.lavender;
        color.fade_out(0.8);
        selection_style.background_color = Some(color);

        let highlights = vec![(self.char_range_to_text_range(&text), selection_style)];

        let styled_text: StyledText = if text.len() == 0 {
            text = self.placeholder.to_string();
            style.color = theme.subtext0;
            StyledText::new(text)
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
